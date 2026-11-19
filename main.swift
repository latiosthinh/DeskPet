// DeskPet — a tiny pixel pet (Mochi, Ember, Pip, Puddle or Sprout) that lives beside your Dock.
//
// It wanders back and forth inside a limited "pen" around its home spot.
// When you type, it pulls out its own laptop and types along. Now and then it
// sweeps the floor or reads a book. It reminds you to drink water, eat your
// main meals and stretch, and falls asleep when you're away.
//
// Drag the pet to move its home. Click it to pet it (or dismiss a reminder).
// Menu-bar pet face → Roam Range controls how far it may travel.

import Cocoa
import ServiceManagement

// MARK: - Pixel art

let pixel: CGFloat = 4                 // screen points per art pixel
let petSize: CGFloat = 16 * pixel      // sprite is 16×16 art pixels
let propRoom: CGFloat = 36             // space kept inside the pen for props held in front

func rgb(_ r: CGFloat, _ g: CGFloat, _ b: CGFloat) -> NSColor {
    NSColor(calibratedRed: r, green: g, blue: b, alpha: 1)
}

/// Colours for props. "w" (the paw on the laptop) comes from the pet's own body colour.
let propPalette: [Character: NSColor] = [
    "k": rgb(0.20, 0.16, 0.24),  // outline
    "g": rgb(0.66, 0.68, 0.74),  // laptop
    "b": rgb(0.45, 0.78, 1.00),  // laptop screen
    "B": rgb(0.85, 0.97, 1.00),  // laptop screen, bright line
    "n": rgb(0.58, 0.38, 0.22),  // broom handle
    "y": rgb(0.96, 0.78, 0.36),  // broom straw
    "r": rgb(0.87, 0.34, 0.38),  // book cover
    "s": rgb(0.85, 0.82, 0.78),  // turning page
    "c": rgb(0.55, 0.82, 1.00),  // water
    "o": rgb(0.93, 0.46, 0.32),  // bowl
    "i": rgb(1.00, 1.00, 1.00),  // rice, book pages
]

let emptyRow = "................"

enum Eyes { case open, closed, happy }
/// Standing on one of the two walking-feet rows, or sitting down.
enum Pose { case stand(Int), sit }

/// A pet design on the 16×16 grid, facing right. Sprite column 14 is the front outline,
/// where props are held; eyes are drawn on rows 6–7 at `eyeCols`.
struct Pet {
    let id: String
    let name: String
    let palette: [Character: NSColor]
    let body: [String]        // 13 rows, eyes left out
    let feet: [String]        // two walking rows
    var sitBase: String? = nil   // a pet without legs keeps this row under it when it sits
    var paw: Character = "w"
    var eyeCols = [9, 12]

    func rows(eyes: Eyes) -> [String] {
        var grid = body.map { Array($0) }
        for c in eyeCols {
            switch eyes {
            case .open:   grid[6][c] = "k"; grid[7][c] = "k"
            case .closed: grid[7][c - 1] = "k"; grid[7][c] = "k"
            case .happy:  grid[6][c] = "k"; grid[7][c - 1] = "k"; grid[7][c + 1] = "k"
            }
        }
        return grid.map { String($0) }
    }

    func frame(_ eyes: Eyes, _ pose: Pose) -> [String] {
        var out = rows(eyes: eyes)
        var drop = 0
        switch pose {
        case .stand(let step): out.append(feet[step % 2])
        case .sit:
            if let base = sitBase { out.append(base) } else { drop = 1 }
        }
        while out.count < 16 { out.append(emptyRow) }
        for _ in 0..<drop { out.insert(emptyRow, at: 0); out.removeLast() }
        return out
    }
}

let pets: [Pet] = [
    Pet(id: "mochi", name: "Mochi", palette: [
            "k": rgb(0.20, 0.16, 0.24), "w": rgb(0.99, 0.95, 0.89),
            "s": rgb(0.89, 0.82, 0.76), "p": rgb(1.00, 0.62, 0.70)],
        body: ["................", "...kk......kk...", "..kppk....kppk..", "..kpwkkkkkkwpk..",
               ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
               ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
               "...kkkkkkkkkk..."],
        feet: ["..kwk.....kwk...", "....kwk.kwk....."]),
    Pet(id: "ember", name: "Ember the fox", palette: [
            "k": rgb(0.23, 0.14, 0.13), "w": rgb(0.95, 0.53, 0.23),
            "s": rgb(0.83, 0.42, 0.16), "c": rgb(1.00, 0.95, 0.86)],
        body: [".....k.....k....", "....kwk...kwk...", "....kwwk.kwwk...", "...kwwwkkkwwwk..",
               "...kwwwwwwwwwwk.", "...kwwwwwwwwwwk.", ".k.kwwwwwwwwwwk.", "kckkwwwwwwwwwwk.",
               "kwkkwwwwccccckk.", "kwwkswwwcccccck.", "kwwksswwwccccck.", ".kkkssswwwwwwk..",
               "....kkkkkkkkk..."],
        feet: ["....kwk...kwk...", ".....kwk.kwk...."]),
    Pet(id: "pip", name: "Pip the penguin", palette: [
            "k": rgb(0.12, 0.13, 0.20), "b": rgb(0.24, 0.29, 0.43), "w": rgb(1.00, 1.00, 1.00),
            "o": rgb(1.00, 0.65, 0.24), "p": rgb(1.00, 0.62, 0.70)],
        body: ["................", "................", ".....kkkkkk.....", "...kkbbbbbbkk...",
               "..kbbbbbbwwwwk..", ".kbbbbbbwwwwwwk.", ".kbbbbbwwwwwwwk.", ".kbbbbbwwwwwwwko",
               ".kbbbbbwwwwwwpk.", ".kbbkbbwwwwwwwk.", ".kbbbkbwwwwwwwk.", "..kbbbkwwwwwwk..",
               "...kkkkkkkkkk..."],
        feet: ["...kook..kook...", "....kook.kook..."]),
    Pet(id: "puddle", name: "Puddle the slime", palette: [
            "k": rgb(0.14, 0.31, 0.42), "c": rgb(0.56, 0.85, 0.95), "H": rgb(1.00, 1.00, 1.00),
            "d": rgb(0.39, 0.71, 0.86), "p": rgb(1.00, 0.62, 0.70)],
        body: ["................", "................", "................", ".....kkkkkk.....",
               "...kkcccccckk...", "..kcHcccccccck..", ".kcHcccccccccck.", ".kcccccccccccck.",
               ".kccccccpccccpk.", ".kcccccccccccck.", ".kdccccccccccck.", ".kddcccccccccck.",
               ".kddddcccccccck."],
        feet: ["..kkkkkkkkkkkk..", ".kkkkkkkkkkkkkk."],
        sitBase: "..kkkkkkkkkkkk..", paw: "c"),
    Pet(id: "sprout", name: "Sprout the seedling", palette: [
            "k": rgb(0.14, 0.25, 0.16), "w": rgb(0.71, 0.88, 0.54), "s": rgb(0.55, 0.76, 0.40),
            "L": rgb(0.36, 0.70, 0.32), "S": rgb(0.25, 0.54, 0.23), "p": rgb(1.00, 0.62, 0.70)],
        body: ["..kkk...kkk.....", ".kLLLk.kLLLk....", "..kLLLkLLLk.....", "..kkkkSkkkkkkk..",
               ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
               ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
               "...kkkkkkkkkk..."],
        feet: ["..kwk.....kwk...", "....kwk.kwk....."]),
]

func pet(withID id: String) -> Pet { pets.first { $0.id == id } ?? pets[0] }

/// Something the pet holds or uses, drawn in front of it.
/// `col` is the sprite column of the prop's left edge (facing right); `lift` is art pixels above the ground.
struct Prop {
    let rows: [String]
    let col: Int
    let lift: Int
}

/// Props sit just in front of the body (sprite column 14 is the body's front outline).
enum Props {
    static let laptopClosed = ["kkkkkkkkk.", "kgggggggk.", "kkkkkkkkk."]

    /// Side view: keyboard on the floor, lid tilted back with its glowing screen facing the pet.
    static func laptopOpen(pawUp: Bool, flicker: Bool) -> [String] {
        [
            "........kk",
            ".......kbk",
            flicker ? ".......kBk" : ".......kbk",
            "......kbk.",
            pawUp ? "kwk...kbk." : ".kwk..kBk.",
            "kkkkkkkkk.",
            "kgggggggk.",
            "kkkkkkkkk.",
        ]
    }

    static func broom(swish: Bool) -> [String] {
        let handle = ["n......", ".n.....", "..n....", "...n..."]
        let straw = ["..kkk..", ".kyyyk.", "kyyyyyk"]
        return handle + (swish ? straw.map { "." + $0.dropLast() } : straw)
    }

    /// An open book held up in front.
    static func book(flip: Bool) -> [String] {
        let page = flip ? "kiiksik" : "kiikiik"
        return [".kk.kk.", page, "kiikiik", "kiikiik", "krrrrrk"]
    }

    /// `left` goes from 1 (full) to 0 (empty).
    static func cup(left: Double) -> [String] {
        let full = max(0, min(3, Int((left * 3).rounded(.up))))
        return ["k...k"] + (0..<3).map { $0 >= 3 - full ? "kccck" : "k...k" } + [".kkk."]
    }

    static func bowl(left: Double) -> [String] {
        let rice = left > 0.5 ? ["..iiiii..", ".iiiiiii."]
                 : left > 0.05 ? [".........", ".iiiiiii."]
                 : [".........", "........."]
        return rice + ["koooooook", ".koooook."]
    }
}

struct Particle {
    var text: String
    var color: NSColor
    var size: CGFloat
    var x: CGFloat
    var y: CGFloat
    var vx: CGFloat
    var vy: CGFloat
    var life: Double
    let maxLife: Double
}

enum Reminder { case water, lunch, dinner, stretch }

// MARK: - Pet view (the window is the pet's pen; the pet moves inside it)

@MainActor
final class PetView: NSView {
    enum Mode { case idle, walking, sitting, typing, cheering, sleeping, sweeping, reading, drinking, eating }

    var mode: Mode = .idle
    var modeStart: TimeInterval = 0
    var modeUntil: TimeInterval = 0
    var petX: CGFloat = 0
    var targetX: CGFloat = 0
    var facingLeft = false
    var pet: Pet = pets[0]
    var range: CGFloat = 80
    var screenFrame: NSRect = .zero

    var clock: TimeInterval = 0
    var now: TimeInterval = 0
    var lastPuff: TimeInterval = 0
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
    let sweepSpeed: CGFloat = 12
    let sleepAfter: Double = 90       // seconds of no input
    let typingLinger: Double = 3      // keep the laptop out this long after the last key

    var minPetX: CGFloat { propRoom }
    var maxPetX: CGFloat { max(minPetX, bounds.width - petSize - propRoom) }
    /// Clickable area of the sprite in view coordinates.
    var petRect: NSRect { NSRect(x: petX + pixel, y: 0, width: petSize - 2 * pixel, height: petSize - 2 * pixel) }
    var petCenterX: CGFloat { petX + petSize / 2 }
    /// Free to show a reminder: not asleep, not being dragged, nothing else being said.
    var canRemind: Bool { mode != .sleeping && !dragging && bubbleText == nil }

    override var isOpaque: Bool { false }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    // MARK: Behaviour

    func setMode(_ m: Mode, for seconds: Double = 0) {
        mode = m
        modeStart = now
        modeUntil = now + seconds
    }

    func say(_ text: String, for seconds: Double) {
        bubbleText = text
        bubbleUntil = now + seconds
    }

    func showPen(for seconds: Double) { showPenUntil = max(showPenUntil, now + seconds) }

    /// Screen x of a sprite column, honouring which way the pet faces.
    func xForColumn(_ col: CGFloat) -> CGFloat {
        petX + (facingLeft ? 15 - col : col) * pixel
    }

    func spawn(_ text: String, _ color: NSColor, at point: NSPoint, size: CGFloat = 14,
               vx: ClosedRange<CGFloat> = -10...10, vy: ClosedRange<CGFloat> = 22...36, life: Double = 1.4) {
        particles.append(Particle(text: text, color: color, size: size, x: point.x, y: point.y,
                                  vx: .random(in: vx), vy: .random(in: vy), life: life, maxLife: life))
    }

    func spawnAboveHead(_ text: String, _ color: NSColor, count: Int, life: Double = 1.4) {
        for _ in 0..<count {
            spawn(text, color, at: NSPoint(x: petCenterX + .random(in: -14...10), y: petSize - 14 + .random(in: -4...4)), life: life)
        }
    }

    func petted() {
        bubbleText = nil   // a click also dismisses a reminder
        setMode(.cheering, for: 1.6)
        spawnAboveHead("♥", NSColor.systemPink, count: 3)
    }

    func remind(_ reminder: Reminder) {
        switch reminder {
        case .water:
            setMode(.drinking, for: 7)
            say("Time for some water 💧", for: 9)
        case .lunch:
            setMode(.eating, for: 9)
            say("Lunch time! Go eat 🍚", for: 12)
        case .dinner:
            setMode(.eating, for: 9)
            say("Dinner time! 🍲", for: 12)
        case .stretch:
            setMode(.cheering, for: 3)
            say("Stretch break? 🙆", for: 9)
        }
    }

    func randomTarget(near x: CGFloat, within reach: CGFloat) -> CGFloat {
        let lo = max(minPetX, x - reach), hi = min(maxPetX, x + reach)
        guard hi > lo else { return x }
        var t = x
        for _ in 0..<6 {
            t = .random(in: lo...hi)
            if abs(t - x) > 20 { break }
        }
        return t
    }

    func decideNext() {
        let canWalk = maxPetX - minPetX > 24
        let r = Double.random(in: 0..<1)
        switch r {
        case ..<0.45 where canWalk:
            targetX = randomTarget(near: petX, within: .greatestFiniteMagnitude)
            setMode(.walking)
        case ..<0.57:
            setMode(.sweeping, for: .random(in: 6...10))
            targetX = randomTarget(near: petX, within: 50)
        case ..<0.70:
            setMode(.reading, for: .random(in: 8...14))
        case ..<0.90:
            setMode(.sitting, for: .random(in: 3...8))
        default:
            setMode(.idle, for: .random(in: 2...4))
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
        let busyWithReminder = mode == .drinking || mode == .eating || mode == .cheering
        if mode != .sleeping && idle > sleepAfter {
            setMode(.sleeping)
        } else if mode == .sleeping && idle < 1 {
            setMode(.idle, for: 1.5)
            say("!", for: 1.2)
        } else if mode != .sleeping && !busyWithReminder && keyIdle < 0.6 {
            if mode != .typing { setMode(.typing) }   // pull the laptop out
            modeUntil = now + typingLinger
        }

        switch mode {
        case .walking:
            if moveToward(targetX, speed: walkSpeed, dt: dt) { setMode(.idle, for: .random(in: 2...5)) }

        case .sweeping:
            if moveToward(targetX, speed: sweepSpeed, dt: dt) { targetX = randomTarget(near: petX, within: 50) }
            if now - lastPuff > 0.35 {
                lastPuff = now
                spawn("·", NSColor.systemBrown.withAlphaComponent(0.8),
                      at: NSPoint(x: xForColumn(19), y: 0),
                      size: 16, vx: -14...14, vy: 6...16, life: 0.9)
            }
            if now >= modeUntil { decideNext() }

        case .sleeping:
            if now - lastPuff > 1.3 {
                lastPuff = now
                spawnAboveHead("z", NSColor.secondaryLabelColor, count: 1, life: 2.2)
            }

        case .eating:
            if now - lastPuff > 0.8 {
                lastPuff = now
                spawn("~", NSColor.secondaryLabelColor, at: NSPoint(x: xForColumn(18), y: 18),
                      size: 12, vx: -3...3, vy: 10...16, life: 1.2)
            }
            if now >= modeUntil { petDoneEating() }

        case .drinking:
            if now >= modeUntil { petDoneEating() }

        case .idle, .sitting, .typing, .cheering, .reading:
            if now >= modeUntil { decideNext() }
        }
        petX = min(max(petX, minPetX), maxPetX)   // never leave the pen

        // Look at the cursor when it comes close (not while using a prop).
        if [.idle, .sitting, .cheering].contains(mode),
           mouse.y < bounds.height + 160, abs(mouse.x - petCenterX) < 220, abs(mouse.x - petCenterX) > 6 {
            facingLeft = mouse.x < petCenterX
        }
    }

    private func petDoneEating() {
        setMode(.cheering, for: 1.2)
        spawnAboveHead("♥", NSColor.systemPink, count: 1)
    }

    /// Walks toward x; returns true on arrival.
    private func moveToward(_ x: CGFloat, speed: CGFloat, dt: Double) -> Bool {
        let dx = x - petX
        let stride = speed * dt
        if abs(dx) <= stride { petX = x; return true }
        facingLeft = dx < 0
        petX += dx > 0 ? stride : -stride
        return false
    }

    func currentFrame() -> [String] {
        let blink = clock.truncatingRemainder(dividingBy: 4.2) < 0.14
        let eyes: Eyes = blink ? .closed : .open
        let beat = Int(clock / 0.16) % 2 == 0
        let t = now - modeStart
        let step = { (period: Double) in Int(self.clock / period) % 2 }
        if dragging { return pet.frame(.happy, .stand(step(0.1))) }
        switch mode {
        case .walking:  return pet.frame(.open, .stand(step(0.18)))
        case .idle:     return pet.frame(eyes, .stand(0))
        case .sitting, .typing, .reading:
                        return pet.frame(eyes, .sit)
        case .cheering: return pet.frame(.happy, beat ? .stand(0) : .sit)
        case .sleeping: return pet.frame(.closed, .sit)
        case .sweeping: return pet.frame(eyes, .stand(step(0.3)))
        case .drinking: return pet.frame(isSipping(t) ? .happy : .open, .sit)
        case .eating:   return pet.frame(step(0.4) == 0 ? .happy : .open, .sit)
        }
    }

    private func isSipping(_ t: Double) -> Bool { t.truncatingRemainder(dividingBy: 1.6) > 1.0 }

    func currentProp() -> Prop? {
        if dragging { return nil }
        let t = now - modeStart
        let progress = modeUntil > modeStart ? min(1, t / (modeUntil - modeStart)) : 0
        switch mode {
        case .typing:
            if t < 0.3 { return Prop(rows: Props.laptopClosed, col: 14, lift: 0) }
            let paw = Int(clock / 0.12) % 2 == 0
            let flicker = Int(clock / 0.5) % 2 == 0
            return Prop(rows: Props.laptopOpen(pawUp: paw, flicker: flicker), col: 14, lift: 0)
        case .sweeping:
            return Prop(rows: Props.broom(swish: Int(clock / 0.3) % 2 == 0), col: 14, lift: 0)
        case .reading:
            return Prop(rows: Props.book(flip: t.truncatingRemainder(dividingBy: 3) < 0.25), col: 13, lift: 3)
        case .drinking:
            return Prop(rows: Props.cup(left: 1 - progress), col: 14, lift: isSipping(t) ? 5 : 3)
        case .eating:
            return Prop(rows: Props.bowl(left: 1 - progress), col: 14, lift: 0)
        default:
            return nil
        }
    }

    // MARK: Drawing

    override func draw(_ dirtyRect: NSRect) {
        NSColor.clear.setFill()
        bounds.fill(using: .copy)

        if dragging || now < showPenUntil { drawPen() }

        // Snap to whole device pixels and skip anti-aliasing so the art has no seams.
        let scale = window?.backingScaleFactor ?? 2
        let drawX = (petX * scale).rounded() / scale
        NSGraphicsContext.current?.shouldAntialias = false

        // Sprite: art row 13 (feet) sits on the ground line, so shift down 2 art pixels.
        drawGrid(currentFrame(), palette: pet.palette, x: drawX, col: 0, bottomY: -2 * pixel)
        if let prop = currentProp() {
            var colors = propPalette
            colors["w"] = pet.palette[pet.paw]
            colors["k"] = pet.palette["k"]
            drawGrid(prop.rows, palette: colors, x: drawX, col: prop.col, bottomY: CGFloat(prop.lift) * pixel)
        }

        NSGraphicsContext.current?.shouldAntialias = true

        for p in particles {
            let alpha = max(0, min(1, p.life / p.maxLife * 1.5))
            let attrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.boldSystemFont(ofSize: p.size),
                .foregroundColor: p.color.withAlphaComponent(alpha),
            ]
            NSAttributedString(string: p.text, attributes: attrs).draw(at: NSPoint(x: p.x, y: p.y))
        }

        if let text = bubbleText { drawBubble(text) }
    }

    /// Draws pixel rows whose left edge is sprite column `col` and whose bottom row sits at `bottomY`.
    private func drawGrid(_ rows: [String], palette: [Character: NSColor], x: CGFloat, col: Int, bottomY: CGFloat) {
        for (r, row) in rows.enumerated() {
            let y = bottomY + CGFloat(rows.count - 1 - r) * pixel
            for (c, ch) in row.enumerated() {
                guard let color = palette[ch] else { continue }
                let spriteCol = col + c
                let drawCol = facingLeft ? 15 - spriteCol : spriteCol
                color.setFill()
                NSRect(x: x + CGFloat(drawCol) * pixel, y: y, width: pixel, height: pixel).fill()
            }
        }
    }

    private func drawPen() {
        let rect = NSRect(x: minPetX, y: 1, width: maxPetX - minPetX + petSize, height: petSize - 4)
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
            setMode(.idle, for: 2)
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
    let cornerMargin: CGFloat = 4
    var breaksOn = true
    var waterOn = true
    var mealsOn = true
    var visible = true

    var lastTick = ProcessInfo.processInfo.systemUptime
    var streakStart = ProcessInfo.processInfo.systemUptime
    /// Each interval is re-picked at random within its range, so reminders don't feel mechanical.
    static let breakRange: ClosedRange<Double> = 40 * 60...60 * 60
    static let waterRange: ClosedRange<Double> = 20 * 60...30 * 60
    var breakDue = Double.random(in: breakRange)
    let breakResetIdle: Double = 5 * 60

    var activeSinceWater: Double = 0
    var waterDue = Double.random(in: waterRange)
    /// Main meals as (name, hour, minute). A meal reminder can fire once a day, up to 2 hours after its time.
    let meals: [(Reminder, String, Int, Int)] = [(.lunch, "lunch", 12, 0), (.dinner, "dinner", 18, 30)]
    var lastMealCheck: TimeInterval = 0

    var demoQueue: [() -> Void] = []
    var nextDemo: TimeInterval = 0

    var screen: NSScreen { NSScreen.screens.first ?? NSScreen.main! }

    func applicationDidFinishLaunching(_ note: Notification) {
        if defaults.object(forKey: "range") != nil { range = CGFloat(defaults.double(forKey: "range")) }
        if defaults.object(forKey: "homeX") != nil { homeX = CGFloat(defaults.double(forKey: "homeX")) }
        if defaults.object(forKey: "breaks") != nil { breaksOn = defaults.bool(forKey: "breaks") }
        if defaults.object(forKey: "water") != nil { waterOn = defaults.bool(forKey: "water") }
        if defaults.object(forKey: "meals") != nil { mealsOn = defaults.bool(forKey: "meals") }
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
        view.pet = pet(withID: defaults.string(forKey: "pet") ?? "mochi")
        view.onHomeMoved = { [weak self] in self?.homeMoved() }
        window.contentView = view
        layout()
        view.petX = (view.minPetX + view.maxPetX) / 2
        view.say("hi!", for: 2)
        if visible { window.orderFrontRegardless() }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.image = Self.menuBarIcon(for: view.pet)
        rebuildMenu()

        // `--demo` cycles through every activity and reminder, for trying the app out.
        if CommandLine.arguments.contains("--demo") {
            let v = view!
            demoQueue = [
                { v.setMode(.typing, for: 6) },
                { v.setMode(.sweeping, for: 6); v.targetX = v.randomTarget(near: v.petX, within: 50) },
                { v.setMode(.reading, for: 6) },
                { v.remind(.water) },
                { v.remind(.lunch) },
                { v.remind(.stretch) },
            ]
            nextDemo = ProcessInfo.processInfo.systemUptime + 2
        }

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
        let width = min(sf.width, 2 * range + petSize + 2 * propRoom)
        let height = petSize + 70
        let home = homeX ?? (sf.maxX - cornerMargin - width / 2)
        let x = min(max(home - width / 2, sf.minX), sf.maxX - width)
        window.setFrame(NSRect(x: x, y: sf.minY + 2, width: width, height: height), display: true)
        view.screenFrame = sf
        view.range = range
    }

    func keepPet(atScreenX absX: CGFloat) {
        view.petX = min(max(absX - window.frame.minX, view.minPetX), view.maxPetX)
    }

    func homeMoved() {
        homeX = window.frame.midX
        defaults.set(Double(homeX!), forKey: "homeX")
    }

    @objc func screensChanged() {
        let absX = window.frame.minX + view.petX
        layout()
        keepPet(atScreenX: absX)
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

        checkReminders(now: now, dt: dt, idle: idle)

        guard visible else { return }
        if !demoQueue.isEmpty && now >= nextDemo {
            demoQueue.removeFirst()()
            nextDemo = now + 7
        }

        let mouse = NSEvent.mouseLocation
        let frame = window.frame
        let demoRunning = !demoQueue.isEmpty || now < nextDemo   // demo ignores real typing and idleness
        view.step(dt: dt, now: now, idle: demoRunning ? 0 : idle, keyIdle: demoRunning ? 99 : keyIdle,
                  mouse: NSPoint(x: mouse.x - frame.minX, y: mouse.y - frame.minY))

        // Only the pet itself catches clicks; the rest of the pen is click-through.
        let petOnScreen = view.petRect.offsetBy(dx: frame.minX, dy: frame.minY)
        window.ignoresMouseEvents = !(view.dragging || petOnScreen.contains(mouse))
        view.needsDisplay = true
    }

    // MARK: Reminders

    func checkReminders(now: TimeInterval, dt: Double, idle: Double) {
        let atDesk = idle < 60
        if atDesk { activeSinceWater += dt }
        if idle > breakResetIdle { streakStart = now }   // a 5-minute pause counts as a break
        guard visible && atDesk && view.canRemind else { return }

        if breaksOn && now - streakStart > breakDue {
            streakStart = now
            breakDue = .random(in: Self.breakRange)
            view.remind(.stretch)
        } else if waterOn && activeSinceWater > waterDue {
            activeSinceWater = 0
            waterDue = .random(in: Self.waterRange)
            view.remind(.water)
        } else if mealsOn && now - lastMealCheck > 30 {
            lastMealCheck = now
            checkMeals()
        }
    }

    func checkMeals() {
        let date = Date()
        let cal = Calendar.current
        let today = cal.startOfDay(for: date)
        for (reminder, key, hour, minute) in meals {
            guard let start = cal.date(bySettingHour: hour, minute: minute, second: 0, of: date) else { continue }
            let lastShown = defaults.object(forKey: "lastMeal.\(key)") as? Date ?? .distantPast
            if date >= start && date < start.addingTimeInterval(2 * 3600) && lastShown < today {
                defaults.set(date, forKey: "lastMeal.\(key)")
                view.remind(reminder)
                return
            }
        }
    }

    // MARK: Menu

    /// The pet in its own colours, cropped to its drawn pixels. 1.5 pt per art pixel
    /// is 3 device pixels on Retina, so it stays crisp at about the height of other icons.
    static func menuBarIcon(for pet: Pet) -> NSImage {
        let all = pet.rows(eyes: .open)
        let used = all.indices.filter { all[$0].contains { $0 != "." } }
        let rows = Array(all[used.first!...used.last!])
        let colsUsed = (0..<16).filter { c in rows.contains { Array($0)[c] != "." } }
        let cols = colsUsed.first!...colsUsed.last!
        let palette = pet.palette
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

        let petItem = NSMenuItem(title: "Pet", action: nil, keyEquivalent: "")
        let petMenu = NSMenu()
        for (i, p) in pets.enumerated() {
            let item = NSMenuItem(title: p.name, action: #selector(pickPet(_:)), keyEquivalent: "")
            item.tag = i
            item.image = Self.menuBarIcon(for: p)
            item.state = p.id == view.pet.id ? .on : .off
            item.target = self
            petMenu.addItem(item)
        }
        petItem.submenu = petMenu
        menu.addItem(petItem)
        menu.addItem(.separator())

        let rangeItem = NSMenuItem(title: "Roam Range", action: nil, keyEquivalent: "")
        let rangeMenu = NSMenu()
        for (i, opt) in rangeOptions.enumerated() {
            let item = NSMenuItem(title: opt.1, action: #selector(pickRange(_:)), keyEquivalent: "")
            item.tag = i
            item.state = opt.0 == range ? .on : .off
            item.target = self
            rangeMenu.addItem(item)
        }
        rangeItem.submenu = rangeMenu
        menu.addItem(rangeItem)
        menu.addItem(NSMenuItem(title: "Show Roam Area", action: #selector(showArea), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Move Home to Corner", action: #selector(centerHome), keyEquivalent: ""))
        menu.addItem(.separator())

        let remindItem = NSMenuItem(title: "Reminders", action: nil, keyEquivalent: "")
        let remindMenu = NSMenu()
        for (title, on, action) in [
            ("Drink Water (every 20–30 min)", waterOn, #selector(toggleWater)),
            ("Main Meals (lunch 12:00, dinner 18:30)", mealsOn, #selector(toggleMeals)),
            ("Stretch Break (every 40–60 min)", breaksOn, #selector(toggleBreaks)),
        ] {
            let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
            item.state = on ? .on : .off
            item.target = self
            remindMenu.addItem(item)
        }
        remindItem.submenu = remindMenu
        menu.addItem(remindItem)

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
        keepPet(atScreenX: absX)
        if view.mode == .walking || view.mode == .sweeping { view.setMode(.idle) }
        view.showPen(for: 2.5)
        rebuildMenu()
    }

    @objc func showArea() { view.showPen(for: 3) }

    @objc func centerHome() {
        let absX = window.frame.minX + view.petX
        homeX = nil
        defaults.removeObject(forKey: "homeX")
        layout()
        keepPet(atScreenX: absX)
        view.showPen(for: 2)
    }

    @objc func toggleBreaks() {
        breaksOn.toggle()
        defaults.set(breaksOn, forKey: "breaks")
        streakStart = ProcessInfo.processInfo.systemUptime
        rebuildMenu()
    }

    @objc func toggleWater() {
        waterOn.toggle()
        defaults.set(waterOn, forKey: "water")
        activeSinceWater = 0
        rebuildMenu()
    }

    @objc func pickPet(_ sender: NSMenuItem) {
        let chosen = pets[sender.tag]
        view.pet = chosen
        defaults.set(chosen.id, forKey: "pet")
        statusItem.button?.image = Self.menuBarIcon(for: chosen)
        view.petted()
        view.say("hi, I'm \(chosen.name.components(separatedBy: " ").first!)!", for: 2.5)
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
