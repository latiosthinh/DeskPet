// DeskPet — a tiny pixel critter that lives on top of your Dock.
//
// It wanders back and forth inside a limited "pen" around its home spot,
// reacts when you type, falls asleep when you're away, and nudges you to
// take a break after long stretches at the keyboard.
//
// Drag the pet to move its home. Click it to pet it.
// Menu-bar pet face → Roam Range controls how far it may travel.

import Cocoa
import ServiceManagement

// MARK: - Pixel art

let pixel: CGFloat = 4                 // screen points per art pixel
let petSize: CGFloat = 16 * pixel      // sprite is 16×16 art pixels

let palette: [Character: NSColor] = [
    "k": NSColor(calibratedRed: 0.20, green: 0.16, blue: 0.24, alpha: 1),  // outline / eyes
    "w": NSColor(calibratedRed: 0.99, green: 0.95, blue: 0.89, alpha: 1),  // body
    "s": NSColor(calibratedRed: 0.89, green: 0.82, blue: 0.76, alpha: 1),  // shading
    "p": NSColor(calibratedRed: 1.00, green: 0.62, blue: 0.70, alpha: 1),  // ears / cheeks
]

// Facing right. 13 body rows; feet and padding are added per frame.
let bodyRows = [
    "................",
    "...kk......kk...",
    "..kppk....kppk..",
    "..kpwkkkkkkwpk..",
    ".kwwwwwwwwwwwwk.",
    ".kwwwwwwwwwwwwk.",
    ".kwwwwwwwkwwkwk.",
    ".kwwwwwwwkwwkwk.",
    ".kwwwwwwpwwwwpk.",
    ".kswwwwwwwwwwwk.",
    ".ksswwwwwwwwwwk.",
    "..ksssswwwwwwk..",
    "...kkkkkkkkkk...",
]
let noEyesRow   = ".kwwwwwwwwwwwwk."
let closedRow   = ".kwwwwwwkkwkkwk."
let happyTopRow = ".kwwwwwwwkwwkwk."
let happyBotRow = ".kwwwwwwkwkkwkk."
let feetA       = "..kwk.....kwk..."
let feetB       = "....kwk.kwk....."
let emptyRow    = "................"

enum Eyes { case open, closed, happy }
enum Feet { case a, b, none }

func makeFrame(_ eyes: Eyes, _ feet: Feet, drop: Int = 0) -> [String] {
    var rows = bodyRows
    switch eyes {
    case .open: break
    case .closed: rows[6] = noEyesRow; rows[7] = closedRow
    case .happy:  rows[6] = happyTopRow; rows[7] = happyBotRow
    }
    switch feet {
    case .a: rows.append(feetA)
    case .b: rows.append(feetB)
    case .none: break
    }
    while rows.count < 16 { rows.append(emptyRow) }
    for _ in 0..<drop { rows.insert(emptyRow, at: 0); rows.removeLast() }
    return rows
}

struct Particle {
    var text: String
    var color: NSColor
    var x: CGFloat
    var y: CGFloat
    var vx: CGFloat
    var vy: CGFloat
    var life: Double
    let maxLife: Double
}

// MARK: - Pet view (the window is the pet's pen; the pet moves inside it)

@MainActor
final class PetView: NSView {
    enum Mode { case idle, walking, sitting, watching, cheering, sleeping }

    var mode: Mode = .idle
    var modeUntil: TimeInterval = 0
    var petX: CGFloat = 0
    var targetX: CGFloat = 0
    var facingLeft = false
    var range: CGFloat = 160
    var screenFrame: NSRect = .zero

    var clock: TimeInterval = 0
    var now: TimeInterval = 0
    var lastZ: TimeInterval = 0
    var particles: [Particle] = []
    var bubbleText: String?
    var bubbleUntil: TimeInterval = 0
    var showPenUntil: TimeInterval = 0

    var dragging = false
    private var dragMoved = false
    private var dragStartMouse: NSPoint = .zero
    private var dragStartOrigin: NSPoint = .zero
    var onHomeMoved: (() -> Void)?

    let walkSpeed: CGFloat = 42       // points per second
    let sleepAfter: Double = 90       // seconds of no input
    let walkSpeedFrame: Double = 0.18

    var maxPetX: CGFloat { max(0, bounds.width - petSize) }
    /// Clickable area of the sprite in view coordinates.
    var petRect: NSRect { NSRect(x: petX + pixel, y: 0, width: petSize - 2 * pixel, height: petSize - 2 * pixel) }
    var petCenterX: CGFloat { petX + petSize / 2 }

    override var isOpaque: Bool { false }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    // MARK: Behaviour

    func say(_ text: String, for seconds: Double) {
        bubbleText = text
        bubbleUntil = now + seconds
    }

    func showPen(for seconds: Double) { showPenUntil = max(showPenUntil, now + seconds) }

    func spawn(_ text: String, _ color: NSColor, count: Int, life: Double = 1.4) {
        for _ in 0..<count {
            particles.append(Particle(
                text: text, color: color,
                x: petCenterX + .random(in: -14...10), y: petSize - 14 + .random(in: -4...4),
                vx: .random(in: -10...10), vy: .random(in: 22...36),
                life: life, maxLife: life))
        }
    }

    func petted() {
        mode = .cheering
        modeUntil = now + 1.6
        spawn("♥", NSColor.systemPink, count: 3)
    }

    func remindBreak() {
        mode = .cheering
        modeUntil = now + 3
        say("Stretch break? 🙆", for: 8)
    }

    func decideNext() {
        let canWalk = range > 0 && maxPetX > 24
        let r = Double.random(in: 0..<1)
        if canWalk && r < 0.55 {
            var t = petX
            for _ in 0..<6 {
                t = .random(in: 0...maxPetX)
                if abs(t - petX) > 24 { break }
            }
            targetX = t
            mode = .walking
        } else if r < 0.85 {
            mode = .sitting
            modeUntil = now + .random(in: 3...8)
        } else {
            mode = .idle
            modeUntil = now + .random(in: 2...4)
        }
    }

    func step(dt: Double, now: TimeInterval, idle: Double, keyIdle: Double, mouse: NSPoint) {
        self.now = now
        clock += dt

        for i in particles.indices {
            particles[i].x += particles[i].vx * dt
            particles[i].y += particles[i].vy * dt
            particles[i].life -= dt
        }
        particles.removeAll { $0.life <= 0 }
        if bubbleText != nil && now > bubbleUntil { bubbleText = nil }

        if dragging { return }

        // React to what the human is doing.
        if mode != .sleeping && idle > sleepAfter {
            mode = .sleeping
            lastZ = now
        } else if mode == .sleeping && idle < 1 {
            mode = .idle
            modeUntil = now + 1.5
            say("!", for: 1.2)
        } else if mode != .sleeping && mode != .cheering && keyIdle < 0.6 {
            mode = .watching
            modeUntil = now + 1.2
        }

        switch mode {
        case .walking:
            let dx = targetX - petX
            let stride = walkSpeed * dt
            facingLeft = dx < 0
            if abs(dx) <= stride {
                petX = targetX
                mode = .idle
                modeUntil = now + .random(in: 2...5)
            } else {
                petX += dx > 0 ? stride : -stride
            }
        case .sleeping:
            if now - lastZ > 1.3 {
                lastZ = now
                spawn("z", NSColor.secondaryLabelColor, count: 1, life: 2.2)
            }
        case .idle, .sitting, .watching, .cheering:
            if now >= modeUntil { decideNext() }
        }
        petX = min(max(petX, 0), maxPetX)   // never leave the pen

        // Look at the cursor when it comes close.
        if mode != .walking && mode != .sleeping,
           mouse.y < bounds.height + 160, abs(mouse.x - petCenterX) < 220, abs(mouse.x - petCenterX) > 6 {
            facingLeft = mouse.x < petCenterX
        }
    }

    func currentFrame() -> [String] {
        let blink = clock.truncatingRemainder(dividingBy: 4.2) < 0.14
        let eyes: Eyes = blink ? .closed : .open
        let beat = Int(clock / 0.16) % 2 == 0
        if dragging { return makeFrame(.happy, Int(clock / 0.1) % 2 == 0 ? .a : .b) }
        switch mode {
        case .walking:  return makeFrame(.open, Int(clock / walkSpeedFrame) % 2 == 0 ? .a : .b)
        case .idle:     return makeFrame(eyes, .a)
        case .sitting:  return makeFrame(eyes, .none, drop: 1)
        case .watching: return makeFrame(.open, .none, drop: Int(clock / 0.25) % 2 == 0 ? 1 : 0)
        case .cheering: return makeFrame(.happy, beat ? .a : .none, drop: beat ? 0 : 1)
        case .sleeping: return makeFrame(.closed, .none, drop: 1)
        }
    }

    // MARK: Drawing

    override func draw(_ dirtyRect: NSRect) {
        NSColor.clear.setFill()
        bounds.fill(using: .copy)

        if dragging || now < showPenUntil { drawPen() }

        // Sprite: art row 13 (feet) sits on the ground line, so shift down 2 art pixels.
        let originY = -2 * pixel
        // Snap to whole device pixels and skip anti-aliasing so the art has no seams.
        let scale = window?.backingScaleFactor ?? 2
        let drawX = (petX * scale).rounded() / scale
        NSGraphicsContext.current?.shouldAntialias = false
        let rows = currentFrame()
        for (r, row) in rows.enumerated() {
            for (c, ch) in row.enumerated() {
                guard let color = palette[ch] else { continue }
                let col = facingLeft ? 15 - c : c
                color.setFill()
                NSRect(x: drawX + CGFloat(col) * pixel, y: originY + CGFloat(15 - r) * pixel,
                       width: pixel, height: pixel).fill()
            }
        }

        NSGraphicsContext.current?.shouldAntialias = true

        for p in particles {
            let alpha = max(0, min(1, p.life / p.maxLife * 1.5))
            let attrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.boldSystemFont(ofSize: 14),
                .foregroundColor: p.color.withAlphaComponent(alpha),
            ]
            NSAttributedString(string: p.text, attributes: attrs).draw(at: NSPoint(x: p.x, y: p.y))
        }

        if let text = bubbleText { drawBubble(text) }
    }

    private func drawPen() {
        let rect = NSRect(x: 1, y: 1, width: bounds.width - 2, height: petSize - 4)
        let path = NSBezierPath(roundedRect: rect, xRadius: 10, yRadius: 10)
        NSColor.systemTeal.withAlphaComponent(0.12).setFill()
        path.fill()
        path.lineWidth = 2
        path.setLineDash([6, 4], count: 2, phase: 0)
        NSColor.systemTeal.withAlphaComponent(0.8).setStroke()
        path.stroke()

        let label = range == 0 ? "stays put" : "roams ±\(Int(range)) pt"
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 10, weight: .semibold),
            .foregroundColor: NSColor.systemTeal,
        ]
        let s = NSAttributedString(string: label, attributes: attrs)
        s.draw(at: NSPoint(x: rect.midX - s.size().width / 2, y: rect.maxY + 2))
    }

    private func drawBubble(_ text: String) {
        let attrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 12, weight: .semibold),
            .foregroundColor: NSColor.black,
        ]
        let s = NSAttributedString(string: text, attributes: attrs)
        let size = s.size()
        var box = NSRect(x: petCenterX - size.width / 2 - 8, y: petSize + 4,
                         width: size.width + 16, height: size.height + 8)
        box.origin.x = min(max(box.origin.x, 2), bounds.width - box.width - 2)

        let path = NSBezierPath(roundedRect: box, xRadius: 8, yRadius: 8)
        let tail = NSBezierPath()
        tail.move(to: NSPoint(x: petCenterX - 5, y: box.minY + 1))
        tail.line(to: NSPoint(x: petCenterX, y: box.minY - 6))
        tail.line(to: NSPoint(x: petCenterX + 5, y: box.minY + 1))
        tail.close()
        NSColor.white.setFill()
        path.fill(); tail.fill()
        NSColor(calibratedWhite: 0.2, alpha: 1).setStroke()
        path.lineWidth = 1.5
        path.stroke()
        s.draw(at: NSPoint(x: box.minX + 8, y: box.minY + 4))
    }

    // MARK: Mouse

    override func mouseDown(with event: NSEvent) {
        dragging = true
        dragMoved = false
        dragStartMouse = NSEvent.mouseLocation
        dragStartOrigin = window?.frame.origin ?? .zero
    }

    override func mouseDragged(with event: NSEvent) {
        guard let window else { return }
        let dx = NSEvent.mouseLocation.x - dragStartMouse.x
        if abs(dx) > 3 { dragMoved = true }
        guard dragMoved else { return }
        var x = dragStartOrigin.x + dx
        x = min(max(x, screenFrame.minX), screenFrame.maxX - window.frame.width)
        window.setFrameOrigin(NSPoint(x: x, y: dragStartOrigin.y))
    }

    override func mouseUp(with event: NSEvent) {
        dragging = false
        if dragMoved {
            mode = .idle
            modeUntil = now + 2
            showPen(for: 1.5)
            say("new home!", for: 1.5)
            onHomeMoved?()
        } else {
            petted()
        }
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
    var view: PetView!
    var statusItem: NSStatusItem!
    let defaults = UserDefaults.standard

    var range: CGFloat = 80
    var homeX: CGFloat?
    let cornerMargin: CGFloat = 12
    var breaksOn = true
    var visible = true

    var lastTick = ProcessInfo.processInfo.systemUptime
    var streakStart = ProcessInfo.processInfo.systemUptime
    var lastNag: TimeInterval = 0
    let breakAfter: Double = 50 * 60
    let nagEvery: Double = 10 * 60
    let breakResetIdle: Double = 5 * 60

    var screen: NSScreen { NSScreen.screens.first ?? NSScreen.main! }

    func applicationDidFinishLaunching(_ note: Notification) {
        if defaults.object(forKey: "range") != nil { range = CGFloat(defaults.double(forKey: "range")) }
        if defaults.object(forKey: "homeX") != nil { homeX = CGFloat(defaults.double(forKey: "homeX")) }
        if defaults.object(forKey: "breaks") != nil { breaksOn = defaults.bool(forKey: "breaks") }
        if defaults.object(forKey: "visible") != nil { visible = defaults.bool(forKey: "visible") }

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

        view = PetView()
        view.onHomeMoved = { [weak self] in self?.homeMoved() }
        window.contentView = view
        layout()
        view.petX = view.maxPetX / 2
        view.say("hi!", for: 2)
        if visible { window.orderFrontRegardless() }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.image = Self.menuBarIcon()
        rebuildMenu()

        let timer = Timer(timeInterval: 1.0 / 30, target: self, selector: #selector(tick),
                          userInfo: nil, repeats: true)
        RunLoop.main.add(timer, forMode: .common)

        NotificationCenter.default.addObserver(self, selector: #selector(screensChanged),
            name: NSApplication.didChangeScreenParametersNotification, object: nil)
    }

    /// Sizes the pen window to the roam range around home, standing on the screen floor.
    /// With no saved home, the pen sits in the bottom-right corner, beside the Dock.
    func layout() {
        let sf = screen.frame
        let width = min(sf.width, 2 * range + petSize)
        let height = petSize + 70
        let home = homeX ?? (sf.maxX - cornerMargin - width / 2)
        let x = min(max(home - width / 2, sf.minX), sf.maxX - width)
        window.setFrame(NSRect(x: x, y: sf.minY + 2, width: width, height: height), display: true)
        view.screenFrame = sf
        view.range = range
    }

    func homeMoved() {
        homeX = window.frame.midX
        defaults.set(Double(homeX!), forKey: "homeX")
    }

    @objc func screensChanged() {
        let absX = window.frame.minX + view.petX
        layout()
        view.petX = min(max(absX - window.frame.minX, 0), view.maxPetX)
    }

    static func secondsSince(_ type: CGEventType) -> Double {
        CGEventSource.secondsSinceLastEventType(.combinedSessionState, eventType: type)
    }

    @objc func tick() {
        let now = ProcessInfo.processInfo.systemUptime
        let dt = min(now - lastTick, 0.1)
        lastTick = now

        let keyIdle = Self.secondsSince(.keyDown)
        let idle = [keyIdle, Self.secondsSince(.mouseMoved), Self.secondsSince(.leftMouseDown),
                    Self.secondsSince(.scrollWheel), Self.secondsSince(.flagsChanged)].min()!

        if idle > breakResetIdle { streakStart = now }
        if breaksOn && visible && now - streakStart > breakAfter && now - lastNag > nagEvery {
            lastNag = now
            view.remindBreak()
        }

        guard visible else { return }
        let mouse = NSEvent.mouseLocation
        let frame = window.frame
        view.step(dt: dt, now: now, idle: idle, keyIdle: keyIdle,
                  mouse: NSPoint(x: mouse.x - frame.minX, y: mouse.y - frame.minY))

        // Only the pet itself catches clicks; the rest of the pen is click-through.
        let petOnScreen = view.petRect.offsetBy(dx: frame.minX, dy: frame.minY)
        window.ignoresMouseEvents = !(view.dragging || petOnScreen.contains(mouse))
        view.needsDisplay = true
    }

    // MARK: Menu

    /// The pet's face in its own colours (cream body, pink ears). 1.5 pt per art pixel
    /// is 3 device pixels on Retina, so it stays crisp at about the height of other icons.
    static func menuBarIcon() -> NSImage {
        let rows = Array(makeFrame(.open, .none)[1...12])   // ears down to the bottom outline
        let cols = 1...14
        let px: CGFloat = 1.5
        let size = NSSize(width: CGFloat(cols.count) * px, height: CGFloat(rows.count) * px)
        let image = NSImage(size: size, flipped: false) { _ in
            NSGraphicsContext.current?.shouldAntialias = false
            for (r, row) in rows.enumerated() {
                for (c, ch) in row.enumerated() where cols.contains(c) {
                    guard let color = palette[ch] else { continue }
                    color.setFill()
                    NSRect(x: CGFloat(c - cols.lowerBound) * px, y: CGFloat(rows.count - 1 - r) * px,
                           width: px, height: px).fill()
                }
            }
            return true
        }
        image.isTemplate = false   // keep the colours instead of tinting to the menu bar
        return image
    }

    func rebuildMenu() {
        let menu = NSMenu()

        let show = NSMenuItem(title: "Show Pet", action: #selector(toggleVisible), keyEquivalent: "")
        show.state = visible ? .on : .off
        menu.addItem(show)
        menu.addItem(.separator())

        let rangeItem = NSMenuItem(title: "Roam Range", action: nil, keyEquivalent: "")
        let sub = NSMenu()
        for (i, opt) in rangeOptions.enumerated() {
            let item = NSMenuItem(title: opt.1, action: #selector(pickRange(_:)), keyEquivalent: "")
            item.tag = i
            item.state = opt.0 == range ? .on : .off
            sub.addItem(item)
        }
        rangeItem.submenu = sub
        menu.addItem(rangeItem)
        menu.addItem(NSMenuItem(title: "Show Roam Area", action: #selector(showArea), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Move Home to Corner", action: #selector(centerHome), keyEquivalent: ""))
        menu.addItem(.separator())

        let breaks = NSMenuItem(title: "Break Reminders (every 50 min)", action: #selector(toggleBreaks), keyEquivalent: "")
        breaks.state = breaksOn ? .on : .off
        menu.addItem(breaks)
        let login = NSMenuItem(title: "Open at Login", action: #selector(toggleLogin), keyEquivalent: "")
        loginItem = login
        refreshLoginState()
        menu.addItem(login)
        menu.addItem(.separator())

        let tip = NSMenuItem(title: "Drag the pet to move its home · click to pet", action: nil, keyEquivalent: "")
        tip.isEnabled = false
        menu.addItem(tip)
        menu.addItem(NSMenuItem(title: "Quit DeskPet", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))

        for item in menu.items where item.action != #selector(NSApplication.terminate(_:)) { item.target = self }
        for item in sub.items { item.target = self }
        menu.delegate = self
        statusItem.menu = menu
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

    @objc func toggleVisible() {
        visible.toggle()
        defaults.set(visible, forKey: "visible")
        if visible { window.orderFrontRegardless(); view.say("hi!", for: 1.5) } else { window.orderOut(nil) }
        rebuildMenu()
    }

    @objc func pickRange(_ sender: NSMenuItem) {
        let absX = window.frame.minX + view.petX
        range = rangeOptions[sender.tag].0
        defaults.set(Double(range), forKey: "range")
        layout()
        view.petX = min(max(absX - window.frame.minX, 0), view.maxPetX)
        if view.mode == .walking { view.mode = .idle; view.modeUntil = 0 }
        view.showPen(for: 2.5)
        rebuildMenu()
    }

    @objc func showArea() { view.showPen(for: 3) }

    @objc func centerHome() {
        homeX = nil
        defaults.removeObject(forKey: "homeX")
        layout()
        view.petX = min(view.petX, view.maxPetX)
        view.showPen(for: 2)
    }

    @objc func toggleBreaks() {
        breaksOn.toggle()
        defaults.set(breaksOn, forKey: "breaks")
        streakStart = ProcessInfo.processInfo.systemUptime
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
