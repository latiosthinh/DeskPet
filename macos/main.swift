// DeskPet for Mac — a tiny pixel pet that lives beside your Dock.
//
// It wanders back and forth inside a limited "pen" around its home spot. When you type, it gets
// busy too. It reminds you to drink water, eat your main meals and stretch, and falls asleep when
// you're away.
//
// Every pet is a module (<id>.deskpet, WebAssembly), the same file the Windows app runs. Mochi,
// Kage and Moona come with the app; more come from the DeskPet website. The shared Rust host
// (macos/core, declared in macos/core/deskpet.h) runs the pet and draws it into a pixel buffer; this
// file is the Mac around it: the window, menu bar, mouse, reminders, settings and pet files.
//
// Drag the pet to move its home. Click it to pet it (or dismiss a reminder). Its face in the
// menu bar opens the menu, as does right-clicking it.

import Cocoa
import ServiceManagement

// MARK: - Sizes shared with the pets (petkit::art)

let petSize: CGFloat = 64    // PET_SIZE: the sprite is 16×16 art pixels of 4 points
let propRoom: CGFloat = 36   // PROP_ROOM: space kept inside the pen for props held in front

// MARK: - What the host tells us (JSON from dp_catalog_json and dp_pet_json)

struct PetInfo: Decodable {
    let id: String
    let name: String
    let shortName: String
    let category: String
    let version: String
    let author: String
    let installed: Bool
    let tallest: Double
}

struct TrickInfo: Decodable {
    let id: String
    let title: String
    let seconds: Double
}

struct RunningPet: Decodable {
    let index: Int
    let id: String
    let name: String
    let shortName: String
    let version: String
    let sweeps: Bool
    let hasPastime: Bool
    let clickTrick: String?
    let rightClickTrick: String?
    let tricks: [TrickInfo]
}

func decodeJSON<T: Decodable>(_ text: UnsafePointer<CChar>) -> T? {
    try? JSONDecoder().decode(T.self, from: Data(String(cString: text).utf8))
}

// MARK: - What we lend the host: text drawing and HTTP

/// Draws one line of text white on black for the host, which keeps it as a coverage mask.
/// The host only asks while drawing a frame, on the main thread.
let drawText: DPTextFn = { text, len, font, px, sink in
    let string = String(decoding: UnsafeBufferPointer(start: text, count: len), as: UTF8.self)
    MainActor.assumeIsolated { rasterize(string, symbol: font == 0, px: Int(px), into: sink) }
}

@MainActor
func rasterize(_ string: String, symbol: Bool, px: Int, into sink: UnsafeMutableRawPointer) {
    let size = CGFloat(max(px, 1))
    let nsFont = symbol ? NSFont.boldSystemFont(ofSize: size) : NSFont.systemFont(ofSize: size, weight: .semibold)
    let line = NSAttributedString(string: string, attributes: [.font: nsFont, .foregroundColor: NSColor.white])
    let box = line.size()
    let w = max(1, Int(box.width.rounded(.up)))
    let h = max(1, Int(box.height.rounded(.up)))
    guard let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0,
                              space: CGColorSpaceCreateDeviceGray(), bitmapInfo: CGImageAlphaInfo.none.rawValue)
    else { return }
    ctx.setFillColor(gray: 0, alpha: 1)
    ctx.fill(CGRect(x: 0, y: 0, width: w, height: h))
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(cgContext: ctx, flipped: false)
    line.draw(at: .zero)
    NSGraphicsContext.restoreGraphicsState()
    guard let data = ctx.data else { return }
    dp_mask_set(sink, Int32(w), Int32(h), data.assumingMemoryBound(to: UInt8.self), ctx.bytesPerRow)
}

/// Holds a download's result while the update check waits for it.
final class Download: @unchecked Sendable {
    var data: Data?
}

/// GETs a URL for the update check (on a background thread); hands a 200 response's body to the host.
let httpGet: DPGetFn = { url, limit, sink in
    guard let address = URL(string: String(cString: url)) else { return 0 }
    var request = URLRequest(url: address, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 30)
    request.setValue("DeskPet", forHTTPHeaderField: "User-Agent")
    let download = Download()
    let done = DispatchSemaphore(value: 0)
    URLSession.shared.dataTask(with: request) { data, response, _ in
        if (response as? HTTPURLResponse)?.statusCode == 200 { download.data = data }
        done.signal()
    }.resume()
    done.wait()
    guard let data = download.data, data.count <= limit else { return 0 }
    data.withUnsafeBytes { raw in
        dp_bytes_set(sink, raw.bindMemory(to: UInt8.self).baseAddress, raw.count)
    }
    return 1
}

let sRGB = CGColorSpace(name: CGColorSpace.sRGB)!

/// An image of 0xAARRGGBB words (premultiplied, top row first), as the host draws them.
func makeImage(_ pixels: [UInt32], width: Int, height: Int) -> CGImage? {
    let data = pixels.withUnsafeBufferPointer { Data(buffer: $0) }
    guard let provider = CGDataProvider(data: data as CFData) else { return nil }
    let info = CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue)
    return CGImage(width: width, height: height, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: width * 4,
                   space: sRGB, bitmapInfo: info, provider: provider, decode: nil, shouldInterpolate: false,
                   intent: .defaultIntent)
}

func isPetFile(_ url: URL) -> Bool { url.pathExtension.lowercased() == "deskpet" }

// MARK: - The pen (the window the pet moves in)

@MainActor
final class PenView: NSView {
    weak var app: AppDelegate?
    var image: CGImage? {
        didSet { needsDisplay = true }
    }

    override init(frame: NSRect) {
        super.init(frame: frame)
        wantsLayer = true
        registerForDraggedTypes([.fileURL])
    }

    required init?(coder: NSCoder) { fatalError("not used") }

    override var isOpaque: Bool { false }
    override var wantsUpdateLayer: Bool { true }
    override func updateLayer() {
        layer?.contents = image
        layer?.contentsGravity = .resize
        layer?.magnificationFilter = .nearest
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    override func mouseDown(with event: NSEvent) { app?.mouseDown() }
    override func mouseDragged(with event: NSEvent) { app?.mouseDragged() }
    override func mouseUp(with event: NSEvent) { app?.mouseUp() }
    override func rightMouseDown(with event: NSEvent) { app?.rightClick(event) }

    // A pet file dropped on the pet is installed.
    func petFiles(_ info: NSDraggingInfo) -> [URL] {
        let urls = info.draggingPasteboard.readObjects(forClasses: [NSURL.self], options: [.urlReadingFileURLsOnly: true]) as? [URL]
        return (urls ?? []).filter(isPetFile)
    }

    override func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation {
        petFiles(sender).isEmpty ? [] : .copy
    }

    override func performDragOperation(_ sender: NSDraggingInfo) -> Bool {
        let files = petFiles(sender)
        app?.install(files)
        return !files.isEmpty
    }
}

// MARK: - App

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    let rangeOptions: [(CGFloat, String)] = [
        (0, "Stay Put"), (80, "Tiny  (±80 pt)"), (160, "Small  (±160 pt)"),
        (300, "Medium  (±300 pt)"), (500, "Large  (±500 pt)"),
    ]

    var window: NSPanel!
    var view: PenView!
    var statusItem: NSStatusItem!
    let defaults = UserDefaults.standard

    /// The shared host (DPHost *): the pet list and the running pet.
    var host: OpaquePointer?
    var pets: [PetInfo] = []
    var pet: RunningPet?
    /// Pet files opened before the app finished starting.
    var waitingFiles: [URL] = []

    var range: CGFloat = 80
    var homeX: CGFloat?
    let cornerMargin: CGFloat = 4
    var breaksOn = true
    var waterOn = true
    var mealsOn = true
    var visible = true

    let launched = ProcessInfo.processInfo.systemUptime
    /// Seconds since the app started: the clock the pet runs on.
    func clock() -> Double { ProcessInfo.processInfo.systemUptime - launched }

    var lastTick: Double = 0
    var streakStart: Double = 0
    /// Each interval is re-picked at random within its range, so reminders don't feel mechanical.
    static let breakRange: ClosedRange<Double> = 40 * 60...60 * 60
    static let waterRange: ClosedRange<Double> = 20 * 60...30 * 60
    var breakDue = Double.random(in: breakRange)
    let breakResetIdle: Double = 5 * 60
    var activeSinceWater: Double = 0
    var waterDue = Double.random(in: waterRange)
    /// Main meals as (reminder, name, hour, minute). A meal reminder can fire once a day, up to 2 hours after its time.
    let meals: [(UInt32, String, Int, Int)] = [(1, "lunch", 12, 0), (2, "dinner", 18, 30)]
    var lastMealCheck: Double = 0

    /// `--demo` and friends: steps to run, each followed by a wait in seconds.
    var demoQueue: [(seconds: Double, run: () -> Void)] = []
    var nextDemo: Double = 0

    var dragMoved = false
    var dragStartMouse: CGFloat = 0
    var dragStartX: CGFloat = 0

    var pixels: [UInt32] = []
    var lastPixels: [UInt32] = []

    var screen: NSScreen { NSScreen.screens.first ?? NSScreen.main! }

    /// Where installed pets live.
    var petsFolder: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("DeskPet", isDirectory: true)
            .appendingPathComponent("Pets", isDirectory: true)
    }

    func applicationDidFinishLaunching(_ note: Notification) {
        if defaults.object(forKey: "range") != nil { range = CGFloat(defaults.double(forKey: "range")) }
        if defaults.object(forKey: "homeX") != nil { homeX = CGFloat(defaults.double(forKey: "homeX")) }
        if defaults.object(forKey: "breaks") != nil { breaksOn = defaults.bool(forKey: "breaks") }
        if defaults.object(forKey: "water") != nil { waterOn = defaults.bool(forKey: "water") }
        if defaults.object(forKey: "meals") != nil { mealsOn = defaults.bool(forKey: "meals") }
        if defaults.object(forKey: "visible") != nil { visible = defaults.bool(forKey: "visible") }

        // `--pet <id>` starts with that pet for this run; `--pets <folder>` also loads the pets in a
        // folder (macos/build.sh makes them in build/pets).
        let args = CommandLine.arguments
        func value(_ flag: String) -> String? {
            guard let i = args.firstIndex(of: flag), i + 1 < args.count else { return nil }
            return args[i + 1]
        }
        let made: OpaquePointer
        if let extra = value("--pets") {
            made = dp_new(petsFolder.path, extra, drawText)
        } else {
            made = dp_new(petsFolder.path, nil, drawText)
        }
        host = made
        pets = catalog()

        window = NSPanel(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel],
                         backing: .buffered, defer: false)
        window.isOpaque = false
        window.backgroundColor = .clear
        window.hasShadow = false
        // Just above the Dock's level, so the Dock's full-screen window never covers the pet.
        window.level = NSWindow.Level(rawValue: Int(CGWindowLevelForKey(.dockWindow)) + 1)
        window.hidesOnDeactivate = false
        window.ignoresMouseEvents = true
        window.collectionBehavior = [.canJoinAllSpaces, .stationary, .ignoresCycle, .fullScreenAuxiliary]
        view = PenView(frame: .zero)
        view.app = self
        window.contentView = view

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        let wanted = value("--pet") ?? defaults.string(forKey: "pet") ?? "mochi"
        let index = pets.firstIndex { $0.id.caseInsensitiveCompare(wanted) == .orderedSame } ?? 0
        start(index)
        layout()
        var state = DPState()
        dp_state(made, &state)
        dp_place_at(made, (state.min_pet_x + state.max_pet_x) / 2)
        say("hi!", 2)
        updateIcon()
        rebuildMenu()
        if visible { render(); window.orderFrontRegardless() }

        // `--demo` cycles through every activity, reminder and trick; `--tricks` plays only the pet's
        // tricks; `--typing` and `--pastime` show just that activity.
        let has = { (flag: String) in args.contains(flag) }
        if let pet, has("--demo") || has("--tricks") || has("--typing") || has("--pastime") {
            if has("--demo") {
                demoQueue = [
                    (6, { dp_demo(made, 0, 0) }),
                    (6, { dp_demo(made, 0, 0) }),
                    (7, { dp_demo(made, 1, 6) }),
                ]
                if pet.sweeps { demoQueue.append((7, { dp_demo(made, 3, 6) })) }
                if pet.hasPastime { demoQueue.append((7, { dp_demo(made, 2, 6) })) }
                demoQueue += [(7, { dp_remind(made, 0) }), (7, { dp_remind(made, 1) }), (7, { dp_remind(made, 3) })]
            }
            if has("--typing") {
                demoQueue.append((9, { dp_demo(made, 1, 8) }))
            } else if has("--pastime") {
                demoQueue.append((9, { dp_demo(made, 2, 8) }))
            } else {
                for (i, trick) in pet.tricks.enumerated() {
                    demoQueue.append((trick.seconds + 0.5, { dp_trick(made, UInt32(i)) }))
                }
            }
            nextDemo = clock() + 2
        }

        lastTick = clock()
        streakStart = clock()
        let timer = Timer(timeInterval: 1.0 / 30, target: self, selector: #selector(tick), userInfo: nil, repeats: true)
        RunLoop.main.add(timer, forMode: .common)

        NotificationCenter.default.addObserver(self, selector: #selector(screensChanged),
            name: NSApplication.didChangeScreenParametersNotification, object: nil)

        install(waitingFiles)
        waitingFiles = []
        // Installed pets are kept up to date from the website: shortly after starting, then once a day.
        checkForUpdates(after: 20)
    }

    // MARK: Pets

    func catalog() -> [PetInfo] {
        guard let host else { return [] }
        return decodeJSON(dp_catalog_json(host)) ?? []
    }

    /// Starts pet `index` (the first pet if that one won't start).
    func start(_ index: Int) {
        guard let host else { return }
        let seed = UInt64(Date().timeIntervalSince1970 * 1_000_000)
        dp_start(host, UInt32(max(0, index)), clock(), seed)
        pet = decodeJSON(dp_pet_json(host))
    }

    /// Swaps in another pet where this one stands; it says hello.
    func switchTo(_ index: Int) {
        guard let host else { return }
        let absX = petScreenX()
        start(index)
        layout()
        keepPet(atScreenX: absX)
        if let pet { defaults.set(pet.id, forKey: "pet") }
        updateIcon()
        dp_hello(host)
        rebuildMenu()
    }

    /// Re-reads the pet list after pets were added, removed or updated. Shows pet `id` if given;
    /// otherwise restarts the pet on screen if its file changed, or swaps in the first pet if it's gone.
    func reload(show id: String?) {
        guard let host else { return }
        let next: Int32
        if let id { next = dp_reload(host, id) } else { next = dp_reload(host, nil) }
        pets = catalog()
        if next >= 0 {
            switchTo(Int(next))
        } else {
            pet = decodeJSON(dp_pet_json(host))
            rebuildMenu()
        }
    }

    func message() -> String {
        guard let host else { return "" }
        return String(cString: dp_message(host))
    }

    func say(_ text: String, _ seconds: Double) {
        guard let host else { return }
        dp_say(host, text, seconds)
    }

    /// Installs pet files (opened in Finder or dropped on the pet) and shows the last one.
    func install(_ urls: [URL]) {
        guard let host else {
            waitingFiles += urls
            return
        }
        for url in urls where isPetFile(url) {
            if dp_install(host, url.path) == 1 {
                reload(show: message())
            } else {
                showProblem("Couldn't add the pet \(url.lastPathComponent)", message())
            }
        }
    }

    func application(_ sender: NSApplication, open urls: [URL]) {
        install(urls)
    }

    func checkForUpdates(after seconds: Double) {
        let folder = petsFolder.path
        DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + seconds) {
            let updated = dp_update(folder, httpGet)
            DispatchQueue.main.async {
                MainActor.assumeIsolated {
                    guard let app = NSApp.delegate as? AppDelegate else { return }
                    if updated > 0 { app.reload(show: nil) }
                    app.checkForUpdates(after: 24 * 60 * 60)
                }
            }
        }
    }

    func showProblem(_ title: String, _ detail: String) {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = detail.isEmpty ? "" : detail.prefix(1).uppercased() + String(detail.dropFirst()) + "."
        alert.alertStyle = .warning
        NSApp.activate(ignoringOtherApps: true)
        alert.runModal()
    }

    // MARK: Layout

    /// Sizes the pen window to the roam range around home, standing on the screen floor.
    /// With no saved home, the pen sits in the bottom-right corner, beside the Dock.
    func layout() {
        guard let host else { return }
        let sf = screen.frame
        let tallest = pets.map { CGFloat($0.tallest) }.max() ?? petSize
        let width = min(sf.width, 2 * range + petSize + 2 * propRoom)
        let height = max(tallest, petSize) + 70
        let home = homeX ?? (sf.maxX - cornerMargin - width / 2)
        let x = min(max(home - width / 2, sf.minX), sf.maxX - width)
        window.setFrame(NSRect(x: x, y: sf.minY + 2, width: width, height: height), display: false)
        dp_pen(host, Double(width), Double(height), Double(range))
    }

    func state() -> DPState {
        var s = DPState()
        if let host { dp_state(host, &s) }
        return s
    }

    func petScreenX() -> CGFloat { window.frame.minX + CGFloat(state().pet_x) }

    func keepPet(atScreenX absX: CGFloat) {
        guard let host else { return }
        dp_place_at(host, Double(absX - window.frame.minX))
    }

    @objc func screensChanged() {
        guard state().dragging == 0 else { return }
        let absX = petScreenX()
        layout()
        keepPet(atScreenX: absX)
        lastPixels = []
    }

    func saveHomeIfMoved() {
        guard let host, state().home_moved != 0 else { return }
        homeX = window.frame.midX
        defaults.set(Double(window.frame.midX), forKey: "homeX")
        dp_home_saved(host)
    }

    // MARK: Frame loop

    static func secondsSince(_ type: CGEventType) -> Double {
        CGEventSource.secondsSinceLastEventType(.combinedSessionState, eventType: type)
    }

    @objc func tick() {
        guard let host else { return }
        let now = clock()
        let dt = min(now - lastTick, 0.1)
        lastTick = now

        let keyIdle = Self.secondsSince(.keyDown)
        let idle = [keyIdle, Self.secondsSince(.mouseMoved), Self.secondsSince(.leftMouseDown),
                    Self.secondsSince(.scrollWheel), Self.secondsSince(.flagsChanged)].min()!

        checkReminders(now: now, dt: dt, idle: idle)

        guard visible else { return }
        if !demoQueue.isEmpty && now >= nextDemo {
            let next = demoQueue.removeFirst()
            next.run()
            nextDemo = now + next.seconds
        }

        let mouse = NSEvent.mouseLocation
        let frame = window.frame
        let demoRunning = !demoQueue.isEmpty || now < nextDemo   // the demo ignores real typing and idleness
        let failed = dp_tick(host, dt, now, demoRunning ? 0 : idle, demoRunning ? 99 : keyIdle,
                             Double(mouse.x - frame.minX), Double(mouse.y - frame.minY))
        if failed != 0 {
            // The pet's module broke: the first built-in pet takes over.
            if let problem = dp_failed(host) { NSLog("DeskPet: %@ stopped: %@", pet?.id ?? "?", String(cString: problem)) }
            let name = pet?.shortName ?? "The pet"
            demoQueue.removeAll()   // its queued tricks were that pet's
            switchTo(0)
            say("\(name) ran into a problem", 3)
        }
        saveHomeIfMoved()

        // Only the pet itself catches clicks; the rest of the pen is click-through.
        let s = state()
        let petOnScreen = NSRect(x: s.rect_x, y: s.rect_y, width: s.rect_w, height: s.rect_h).offsetBy(dx: frame.minX, dy: frame.minY)
        window.ignoresMouseEvents = !(s.dragging != 0 || petOnScreen.contains(mouse))
        render()
    }

    /// Draws a frame, and shows it unless nothing changed.
    func render() {
        guard let host else { return }
        let scale = window.backingScaleFactor
        let size = window.frame.size
        let w = max(1, Int((size.width * scale).rounded()))
        let h = max(1, Int((size.height * scale).rounded()))
        if pixels.count != w * h {
            pixels = [UInt32](repeating: 0, count: w * h)
            lastPixels = []
        }
        pixels.withUnsafeMutableBufferPointer { buf in
            dp_draw(host, buf.baseAddress!, Int32(w), Int32(h), Double(scale))
        }
        if pixels == lastPixels { return }
        lastPixels = pixels
        view.image = makeImage(pixels, width: w, height: h)
    }

    // MARK: Mouse

    func mouseDown() {
        guard let host else { return }
        dragMoved = false
        dragStartMouse = NSEvent.mouseLocation.x
        dragStartX = window.frame.minX
        dp_begin_drag(host)
    }

    func mouseDragged() {
        let dx = NSEvent.mouseLocation.x - dragStartMouse
        if abs(dx) > 3 { dragMoved = true }
        guard dragMoved else { return }
        let sf = screen.frame
        let x = min(max(dragStartX + dx, sf.minX), sf.maxX - window.frame.width)
        window.setFrameOrigin(NSPoint(x: x, y: window.frame.minY))
    }

    func mouseUp() {
        guard let host else { return }
        dp_end_drag(host, dragMoved ? 1 : 0)
        saveHomeIfMoved()
    }

    /// A right-click on the pet does its right-click trick if it has one; otherwise (or with Shift) shows the menu.
    func rightClick(_ event: NSEvent) {
        guard let host else { return }
        if event.modifierFlags.contains(.shift) || dp_right_click(host) == 0 {
            NSMenu.popUpContextMenu(makeMenu(), with: event, for: view)
        }
    }

    // MARK: Reminders

    func checkReminders(now: Double, dt: Double, idle: Double) {
        guard let host else { return }
        let atDesk = idle < 60
        if atDesk { activeSinceWater += dt }
        if idle > breakResetIdle { streakStart = now }   // a 5-minute pause counts as a break
        guard visible && atDesk && state().can_remind != 0 else { return }

        if breaksOn && now - streakStart > breakDue {
            streakStart = now
            breakDue = .random(in: Self.breakRange)
            dp_remind(host, 3)
        } else if waterOn && activeSinceWater > waterDue {
            activeSinceWater = 0
            waterDue = .random(in: Self.waterRange)
            dp_remind(host, 0)
        } else if mealsOn && now - lastMealCheck > 30 {
            lastMealCheck = now
            checkMeals()
        }
    }

    func checkMeals() {
        guard let host else { return }
        let date = Date()
        let cal = Calendar.current
        let today = cal.startOfDay(for: date)
        for (reminder, key, hour, minute) in meals {
            guard let start = cal.date(bySettingHour: hour, minute: minute, second: 0, of: date) else { continue }
            let lastShown = defaults.object(forKey: "lastMeal.\(key)") as? Date ?? .distantPast
            if date >= start && date < start.addingTimeInterval(2 * 3600) && lastShown < today {
                defaults.set(date, forKey: "lastMeal.\(key)")
                dp_remind(host, reminder)
                return
            }
        }
    }

    // MARK: Menu

    /// Pet `index` in its own colours, cropped and centred in a square `points` wide.
    func icon(_ index: Int, points: CGFloat) -> NSImage {
        let image = NSImage(size: NSSize(width: points, height: points))
        guard let host else { return image }
        let size = Int(points * 2)   // sharp on Retina
        var px = [UInt32](repeating: 0, count: size * size)
        px.withUnsafeMutableBufferPointer { dp_icon(host, UInt32(index), Int32(size), $0.baseAddress!) }
        guard let cg = makeImage(px, width: size, height: size) else { return image }
        let out = NSImage(cgImage: cg, size: NSSize(width: points, height: points))
        out.isTemplate = false   // keep the colours instead of tinting to the menu bar
        return out
    }

    func updateIcon() {
        statusItem.button?.image = icon(pet?.index ?? 0, points: 18)
        statusItem.button?.toolTip = "DeskPet · \(pet?.shortName ?? "")"
    }

    func rebuildMenu() {
        let menu = makeMenu()
        menu.delegate = self
        statusItem.menu = menu
    }

    func item(_ title: String, _ action: Selector?, tag: Int = 0, on: Bool = false) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
        item.target = self
        item.tag = tag
        item.state = on ? .on : .off
        return item
    }

    func note(_ title: String) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
        item.isEnabled = false
        return item
    }

    func submenu(_ title: String, _ items: [NSMenuItem]) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
        let menu = NSMenu()
        for i in items { menu.addItem(i) }
        item.submenu = menu
        return item
    }

    func makeMenu() -> NSMenu {
        let menu = NSMenu()
        menu.addItem(item("Show Pet", #selector(toggleVisible), on: visible))

        var petItems: [NSMenuItem] = []
        for (i, p) in pets.enumerated() {
            if i > 0 && p.category != pets[i - 1].category { petItems.append(.separator()) }   // grouped by category
            let entry = item(p.name, #selector(pickPet(_:)), tag: i, on: i == pet?.index)
            entry.image = icon(i, points: 16)
            petItems.append(entry)
        }
        petItems.append(.separator())
        petItems.append(item("Get More Pets…", #selector(getMorePets)))
        if let pet, pets.indices.contains(pet.index), pets[pet.index].installed {
            petItems.append(item("Remove \(pet.name)…", #selector(removePet)))
        }
        menu.addItem(submenu("Pet", petItems))

        if let pet, !pet.tricks.isEmpty {
            var trickItems = [item("Surprise Me", #selector(randomTrick)), .separator()]
            for (i, trick) in pet.tricks.enumerated() {
                trickItems.append(item(trick.title, #selector(pickTrick(_:)), tag: i))
            }
            menu.addItem(submenu("Tricks", trickItems))
        }
        menu.addItem(.separator())

        let ranges = rangeOptions.enumerated().map { i, opt in item(opt.1, #selector(pickRange(_:)), tag: i, on: opt.0 == range) }
        menu.addItem(submenu("Roam Range", ranges))
        menu.addItem(item("Show Roam Area", #selector(showArea)))
        menu.addItem(item("Move Home to Corner", #selector(centerHome)))
        menu.addItem(.separator())

        menu.addItem(submenu("Reminders", [
            item("Drink Water (every 20–30 min)", #selector(toggleWater), on: waterOn),
            item("Main Meals (lunch 12:00, dinner 18:30)", #selector(toggleMeals), on: mealsOn),
            item("Stretch Break (every 40–60 min)", #selector(toggleBreaks), on: breaksOn),
        ]))
        let login = item("Open at Login", #selector(toggleLogin))
        loginItem = login
        refreshLoginState()
        menu.addItem(login)
        menu.addItem(.separator())

        if let pet, let click = pet.clickTrick, let right = pet.rightClickTrick {
            // A pet whose clicks do tricks (Moona) says what they do, and how to reach this menu.
            menu.addItem(note("Click: \(click) · right-click: \(right)"))
            menu.addItem(note("Drag to move its home · Shift+right-click: this menu"))
        } else {
            menu.addItem(note("Drag the pet to move its home · click to pet"))
        }
        menu.addItem(NSMenuItem(title: "Quit DeskPet", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))
        return menu
    }

    // MARK: Open at login

    var loginItem: NSMenuItem?

    /// The setting can also change in System Settings → General → Login Items, so re-read it each time.
    func menuWillOpen(_ menu: NSMenu) { refreshLoginState() }

    func refreshLoginState() {
        switch SMAppService.mainApp.status {
        case .enabled:          loginItem?.state = .on
        case .requiresApproval: loginItem?.state = .mixed
        default:                loginItem?.state = .off
        }
    }

    @objc func toggleLogin() {
        let service = SMAppService.mainApp
        do {
            if service.status == .enabled || service.status == .requiresApproval {
                try service.unregister()
            } else {
                try service.register()
            }
        } catch {
            let alert = NSAlert()
            alert.messageText = "Couldn't change Open at Login"
            alert.informativeText = "\(error.localizedDescription)\n\nYou can add DeskPet yourself in System Settings → General → Login Items."
            NSApp.activate(ignoringOtherApps: true)
            alert.runModal()
        }
        if service.status == .requiresApproval {
            let alert = NSAlert()
            alert.messageText = "Allow DeskPet in Login Items"
            alert.informativeText = "macOS needs you to switch DeskPet on in System Settings → General → Login Items."
            alert.addButton(withTitle: "Open Login Items")
            alert.addButton(withTitle: "Later")
            NSApp.activate(ignoringOtherApps: true)
            if alert.runModal() == .alertFirstButtonReturn { SMAppService.openSystemSettingsLoginItems() }
        }
        refreshLoginState()
    }

    // MARK: Menu commands

    @objc func toggleVisible() {
        visible.toggle()
        defaults.set(visible, forKey: "visible")
        if visible {
            say("hi!", 1.5)
            lastPixels = []
            render()
            window.orderFrontRegardless()
        } else {
            window.orderOut(nil)
        }
        rebuildMenu()
    }

    @objc func pickPet(_ sender: NSMenuItem) {
        switchTo(sender.tag)
    }

    @objc func getMorePets() {
        guard let host, let url = URL(string: String(cString: dp_site(host))) else { return }
        NSWorkspace.shared.open(url)
    }

    @objc func removePet() {
        guard let host, let pet, pets.indices.contains(pet.index), pets[pet.index].installed else { return }
        let alert = NSAlert()
        alert.messageText = "Remove \(pet.name)?"
        alert.informativeText = "You can download it again from the DeskPet website."
        alert.addButton(withTitle: "Remove")
        alert.addButton(withTitle: "Cancel")
        NSApp.activate(ignoringOtherApps: true)
        guard alert.runModal() == .alertFirstButtonReturn else { return }
        if dp_remove(host, UInt32(pet.index)) == 1 {
            reload(show: nil)
        } else {
            showProblem("Couldn't remove \(pet.name)", message())
        }
    }

    @objc func pickTrick(_ sender: NSMenuItem) {
        guard let host else { return }
        dp_trick(host, UInt32(sender.tag))
    }

    @objc func randomTrick() {
        guard let host else { return }
        dp_surprise(host)
    }

    @objc func pickRange(_ sender: NSMenuItem) {
        guard let host else { return }
        let absX = petScreenX()
        range = rangeOptions[sender.tag].0
        defaults.set(Double(range), forKey: "range")
        layout()
        keepPet(atScreenX: absX)
        dp_stop_roaming(host)
        dp_show_pen(host, 2.5)
        rebuildMenu()
    }

    @objc func showArea() {
        guard let host else { return }
        dp_show_pen(host, 3)
    }

    @objc func centerHome() {
        guard let host else { return }
        let absX = petScreenX()
        homeX = nil
        defaults.removeObject(forKey: "homeX")
        layout()
        keepPet(atScreenX: absX)
        dp_show_pen(host, 2)
    }

    @objc func toggleBreaks() {
        breaksOn.toggle()
        defaults.set(breaksOn, forKey: "breaks")
        streakStart = clock()
        rebuildMenu()
    }

    @objc func toggleWater() {
        waterOn.toggle()
        defaults.set(waterOn, forKey: "water")
        activeSinceWater = 0
        rebuildMenu()
    }

    @objc func toggleMeals() {
        mealsOn.toggle()
        defaults.set(mealsOn, forKey: "meals")
        rebuildMenu()
    }
}

MainActor.assumeIsolated {
    let app = NSApplication.shared
    let delegate = AppDelegate()
    app.delegate = delegate
    app.setActivationPolicy(.accessory)
    app.run()
}
