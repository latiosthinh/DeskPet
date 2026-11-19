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
    "i": rgb(1.00, 1.00, 1.00),  // rice, book pages, egg white, marshmallow
    "d": rgb(0.30, 0.30, 0.35),  // frying pan
    "C": rgb(0.80, 0.94, 1.00),  // water surface
    "P": rgb(1.00, 0.62, 0.76),  // petals, butterfly, ice cream
    "G": rgb(0.33, 0.64, 0.30),  // stems
    "N": rgb(0.40, 0.26, 0.15),  // dark soil
    "W": rgb(0.95, 0.96, 0.99),  // cloud
    "T": rgb(0.82, 0.64, 0.42),  // cardboard
    "U": rgb(0.66, 0.49, 0.31),  // cardboard tape and flaps
]

let emptyRow = "................"

enum Eyes { case open, closed, happy }
/// Standing on one of the two walking-feet rows, or sitting down.
enum Pose { case stand(Int), sit }

enum TypingStyle { case laptop, cooking, swimming, readingGlasses, digging }

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
    var tricks: [Trick] = []
    /// What the pet does while you type.
    var typing: TypingStyle = .laptop
    var sweeps = false
    var readsWithGlasses = false

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
        feet: ["..kwk.....kwk...", "....kwk.kwk....."],
        tricks: [.yarn, .boxNap, .laser, .fishing], sweeps: true),
    Pet(id: "ember", name: "Ember the fox", palette: [
            "k": rgb(0.23, 0.14, 0.13), "w": rgb(0.95, 0.53, 0.23),
            "s": rgb(0.83, 0.42, 0.16), "c": rgb(1.00, 0.95, 0.86)],
        body: [".....k.....k....", "....kwk...kwk...", "....kwwk.kwwk...", "...kwwwkkkwwwk..",
               "...kwwwwwwwwwwk.", "...kwwwwwwwwwwk.", ".k.kwwwwwwwwwwk.", "kckkwwwwwwwwwwk.",
               "kwkkwwwwccccckk.", "kwwkswwwcccccck.", "kwwksswwwccccck.", ".kkkssswwwwwwk..",
               "....kkkkkkkkk..."],
        feet: ["....kwk...kwk...", ".....kwk.kwk...."],
        tricks: [.firework, .flashlight, .flipEgg, .campfire], typing: .cooking),
    Pet(id: "pip", name: "Pip the penguin", palette: [
            "k": rgb(0.12, 0.13, 0.20), "b": rgb(0.24, 0.29, 0.43), "w": rgb(1.00, 1.00, 1.00),
            "o": rgb(1.00, 0.65, 0.24), "p": rgb(1.00, 0.62, 0.70)],
        body: ["................", "................", ".....kkkkkk.....", "...kkbbbbbbkk...",
               "..kbbbbbbwwwwk..", ".kbbbbbbwwwwwwk.", ".kbbbbbwwwwwwwk.", ".kbbbbbwwwwwwwko",
               ".kbbbbbwwwwwwpk.", ".kbbkbbwwwwwwwk.", ".kbbbkbwwwwwwwk.", "..kbbbkwwwwwwk..",
               "...kkkkkkkkkk..."],
        feet: ["...kook..kook...", "....kook.kook..."],
        tricks: [.hideInWater, .spitWater, .swim, .bubbles], typing: .swimming),
    Pet(id: "puddle", name: "Puddle the slime", palette: [
            "k": rgb(0.14, 0.31, 0.42), "c": rgb(0.56, 0.85, 0.95), "H": rgb(1.00, 1.00, 1.00),
            "d": rgb(0.39, 0.71, 0.86), "p": rgb(1.00, 0.62, 0.70)],
        body: ["................", "................", "................", ".....kkkkkk.....",
               "...kkcccccckk...", "..kcHcccccccck..", ".kcHcccccccccck.", ".kcccccccccccck.",
               ".kccccccpccccpk.", ".kcccccccccccck.", ".kdccccccccccck.", ".kddcccccccccck.",
               ".kddddcccccccck."],
        feet: ["..kkkkkkkkkkkk..", ".kkkkkkkkkkkkkk."],
        sitBase: "..kkkkkkkkkkkk..", paw: "c",
        tricks: [.iceCream, .twin, .colorShift, .popFood], typing: .readingGlasses, readsWithGlasses: true),
    Pet(id: "sprout", name: "Sprout the seedling", palette: [
            "k": rgb(0.14, 0.25, 0.16), "w": rgb(0.71, 0.88, 0.54), "s": rgb(0.55, 0.76, 0.40),
            "L": rgb(0.36, 0.70, 0.32), "S": rgb(0.25, 0.54, 0.23), "p": rgb(1.00, 0.62, 0.70)],
        body: ["..kkk...kkk.....", ".kLLLk.kLLLk....", "..kLLLkLLLk.....", "..kkkkSkkkkkkk..",
               ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.", ".kwwwwwwwwwwwwk.",
               ".kwwwwwwpwwwwpk.", ".kswwwwwwwwwwwk.", ".ksswwwwwwwwwwk.", "..ksssswwwwwwk..",
               "...kkkkkkkkkk..."],
        feet: ["..kwk.....kwk...", "....kwk.kwk....."],
        tricks: [.giveFlower, .shower, .plantSelf, .growUp, .chaseButterfly], typing: .digging),
]

func pet(withID id: String) -> Pet { pets.first { $0.id == id } ?? pets[0] }

/// Something the pet holds or uses, drawn in front of it.
/// `col` is the sprite column of the prop's left edge (facing right); `lift` is art pixels above the ground.
struct Prop {
    let rows: [String]
    let col: CGFloat
    let lift: CGFloat
}

/// A grid drawn at a fixed spot in the pen (in points), not attached to the pet.
struct FreeSprite {
    let rows: [String]
    let x: CGFloat
    let y: CGFloat
    var px: CGFloat = pixel
}

/// A second copy of the pet (Puddle's twin), centred at `centerX`.
struct Twin {
    let frame: [String]
    let centerX: CGFloat
    let scale: CGFloat
    let yOffset: CGFloat
    let facingLeft: Bool
}

/// Everything drawn for one moment: the pet, how it is placed, and what is around it.
struct Look {
    var frame: [String]
    var palette: [Character: NSColor]
    var yOffset: CGFloat = 0        // art pixels; negative sinks into water or soil
    var scale: CGFloat = 1
    var props: [Prop] = []          // held or placed in front of the pet
    var free: [FreeSprite] = []
    var twin: Twin? = nil
    var beam: (origin: NSPoint, angle: CGFloat)? = nil   // flashlight light
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
    var ay: CGFloat = 0             // gravity (negative falls)
}

enum Reminder { case water, lunch, dinner, stretch }

// MARK: - Pet view (the window is the pet's pen; the pet moves inside it)

@MainActor
final class PetView: NSView {
    enum Mode { case idle, walking, sitting, typing, cheering, sleeping, sweeping, reading, drinking, eating, trick }

    var mode: Mode = .idle
    var modeStart: TimeInterval = 0
    var modeUntil: TimeInterval = 0
    var petX: CGFloat = 0
    var targetX: CGFloat = 0
    var facingLeft = false
    var pet: Pet = pets[0]

    // Trick state
    var trick: Trick?
    var lastTrick: Trick?
    var trickFlags: Set<Int> = []
    var trickTimers: [Int: TimeInterval] = [:]
    var trickStartFacing = false
    var burstCycle = -1
    var poolLeft: CGFloat = 0
    var poolWidth: CGFloat = 0
    var twinCenterX: CGFloat = 0
    var foodKind = 0
    var chaseMoving = false
    var bubbleScale: CGFloat = 1
    var dotX: CGFloat = 0
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
               vx: ClosedRange<CGFloat> = -10...10, vy: ClosedRange<CGFloat> = 22...36, life: Double = 1.4,
               ay: CGFloat = 0) {
        particles.append(Particle(text: text, color: color, size: size, x: point.x, y: point.y,
                                  vx: .random(in: vx), vy: .random(in: vy), life: life, maxLife: life, ay: ay))
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
        // About a third of the time, the pet does one of its own tricks.
        if !pet.tricks.isEmpty && Double.random(in: 0..<1) < 0.33 {
            let choices = pet.tricks.filter { $0 != lastTrick }
            startTrick(choices.randomElement() ?? pet.tricks[0])
            return
        }
        let canWalk = maxPetX - minPetX > 24
        let r = Double.random(in: 0..<1)
        switch r {
        case ..<0.45 where canWalk:
            targetX = randomTarget(near: petX, within: .greatestFiniteMagnitude)
            setMode(.walking)
        case ..<0.57 where pet.sweeps:
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
            particles[i].vy += particles[i].ay * dt
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
            if mode != .typing { beginTyping() }   // pull out the laptop, pan, shovel…
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

        case .trick:
            if let trick { stepTrick(trick, dt: dt) }
            if now >= modeUntil { trick = nil; decideNext() }

        case .typing:
            stepTyping(dt: dt)
            if now >= modeUntil { decideNext() }

        case .idle, .sitting, .cheering, .reading:
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
    func moveToward(_ x: CGFloat, speed: CGFloat, dt: Double) -> Bool {
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
        case .trick:    return pet.frame(eyes, .sit)
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
            return Prop(rows: Props.book(flip: t.truncatingRemainder(dividingBy: 3) < 0.25), col: 13, lift: bookLift)
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

        let look = currentLook()
        bubbleScale = look.scale   // a grown pet gets its speech bubble above its head
        if let beam = look.beam { drawBeam(from: beam.origin, angle: beam.angle) }

        // Skip anti-aliasing so the art has no seams; positions are snapped to device pixels.
        NSGraphicsContext.current?.shouldAntialias = false
        var colors = propPalette
        colors["w"] = look.palette[pet.paw]
        colors["k"] = look.palette["k"]

        if let twin = look.twin {
            drawSprite(twin.frame, palette: look.palette, centerX: twin.centerX, scale: twin.scale,
                       yOffset: twin.yOffset, mirrored: twin.facingLeft)
        }
        drawSprite(look.frame, palette: look.palette, centerX: petCenterX, scale: look.scale,
                   yOffset: look.yOffset, mirrored: facingLeft)
        for prop in look.props {
            drawGrid(prop.rows, palette: colors, x: snap(petX), col: prop.col, bottomY: prop.lift * pixel,
                     px: pixel, mirrored: facingLeft)
        }
        for f in look.free {
            drawGrid(f.rows, palette: colors, x: snap(f.x), col: 0, bottomY: snap(f.y), px: f.px, mirrored: false)
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

    func snap(_ v: CGFloat) -> CGFloat {
        let scale = window?.backingScaleFactor ?? 2
        return (v * scale).rounded() / scale
    }

    /// Draws the 16×16 pet sprite scaled around its centre, standing on the ground.
    /// Art row 13 (feet) sits on the ground line, so the sprite starts 2 art pixels below it.
    private func drawSprite(_ rows: [String], palette: [Character: NSColor], centerX: CGFloat, scale: CGFloat,
                            yOffset: CGFloat, mirrored: Bool) {
        let px = max(0.5, (pixel * scale * 2).rounded() / 2)
        drawGrid(rows, palette: palette, x: snap(centerX - 8 * px), col: 0,
                 bottomY: snap(-2 * px + yOffset * pixel), px: px, mirrored: mirrored)
    }

    /// Draws pixel rows whose left edge is sprite column `col` and whose bottom row sits at `bottomY`.
    /// Mirrored grids flip around the 16-column sprite, so props stay in front of a pet facing left.
    private func drawGrid(_ rows: [String], palette: [Character: NSColor], x: CGFloat, col: CGFloat,
                          bottomY: CGFloat, px: CGFloat = pixel, mirrored: Bool) {
        for (r, row) in rows.enumerated() {
            let y = bottomY + CGFloat(rows.count - 1 - r) * px
            for (c, ch) in row.enumerated() {
                guard let color = palette[ch] else { continue }
                let spriteCol = col + CGFloat(c)
                let drawCol = mirrored ? 15 - spriteCol : spriteCol
                color.setFill()
                NSRect(x: snap(x + drawCol * px), y: y, width: px, height: px).fill()
            }
        }
    }

    private func drawBeam(from p: NSPoint, angle: CGFloat) {
        let dir: CGFloat = facingLeft ? -1 : 1
        let length: CGFloat = 96, spread: CGFloat = 0.34
        let path = NSBezierPath()
        path.move(to: p)
        path.line(to: NSPoint(x: p.x + dir * length * cos(angle - spread), y: p.y + length * sin(angle - spread)))
        path.line(to: NSPoint(x: p.x + dir * length * cos(angle + spread), y: p.y + length * sin(angle + spread)))
        path.close()
        let light = NSColor(calibratedRed: 1, green: 0.93, blue: 0.55, alpha: 0.5)
        NSGradient(starting: light, ending: light.withAlphaComponent(0))?.draw(in: path, angle: facingLeft ? 180 : 0)
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
        var box = NSRect(x: petCenterX - size.width / 2 - 8, y: petSize * bubbleScale + 4,
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

// MARK: - Tricks (each pet's own activities)

enum Trick: CaseIterable {
    case firework, flashlight, flipEgg, campfire               // Ember
    case hideInWater, spitWater, swim, bubbles                 // Pip
    case iceCream, twin, colorShift, popFood                   // Puddle
    case giveFlower, shower, plantSelf, growUp, chaseButterfly // Sprout
    case yarn, boxNap, laser, fishing                          // Mochi

    var duration: Double {
        switch self {
        case .firework: 8
        case .flashlight: 8
        case .flipEgg: 9
        case .campfire: 10
        case .hideInWater: 9.5
        case .spitWater: 6
        case .swim: 10
        case .bubbles: 7
        case .iceCream: 9
        case .twin: 11
        case .colorShift: 8
        case .popFood: 7
        case .giveFlower: 7
        case .shower: 9
        case .plantSelf: 10
        case .growUp: 9
        case .chaseButterfly: 12
        case .yarn: 10
        case .boxNap: 10
        case .laser: 10
        case .fishing: 11
        }
    }

    var title: String {
        switch self {
        case .firework: "Light Fireworks"
        case .flashlight: "Explore with a Flashlight"
        case .flipEgg: "Flip an Egg"
        case .campfire: "Toast a Marshmallow"
        case .hideInWater: "Hide in the Water"
        case .spitWater: "Spit Water"
        case .swim: "Go for a Swim"
        case .bubbles: "Blow Bubbles"
        case .iceCream: "Eat Ice Cream"
        case .twin: "Split in Two and Play"
        case .colorShift: "Change Colour"
        case .popFood: "Pop Out a Snack"
        case .giveFlower: "Give You a Flower"
        case .shower: "Take a Shower"
        case .plantSelf: "Plant Itself"
        case .growUp: "Grow Up"
        case .chaseButterfly: "Chase a Butterfly"
        case .yarn: "Play with Yarn"
        case .boxNap: "Nap in a Box"
        case .laser: "Chase a Laser Dot"
        case .fishing: "Go Fishing"
        }
    }
}

/// Pixel art for tricks, facing right, coloured from `propPalette`.
enum TrickArt {
    static let rocket = [".k.", "krk", "krk", "kyk", ".n."]
    static let flashlight = ["kkkkk", "kgggy", "kkkkk"]
    static let pan = ["....kkkkkk", "nnnnkddddk"]
    static let eggUp = [".iyi.", "iiiii"]
    static let eggDown = [".iii.", "iiiii"]
    static let fireA = ["...y...", "..yoy..", ".yooy..", ".yoooy.", "nknnnkn"]
    static let fireB = ["....y..", "..yoy..", "..yooy.", ".yoooy.", "nknnnkn"]
    static func stick(toasted: Bool) -> [String] {
        let m: Character = toasted ? "y" : "i"
        return ["....\(m)\(m)", "...n\(m)\(m)", "..n...", ".n....", "n....."]
    }
    static func pool(_ width: Int) -> [String] {
        ["." + String(repeating: "C", count: width - 2) + ".",
         String(repeating: "c", count: width), String(repeating: "c", count: width)]
    }
    static func iceCream(left: Double) -> [String] {
        let scoop = left > 0.66 ? ["..PP.", ".PPPP", "PPPPP"]
                  : left > 0.33 ? [".....", ".PPP.", "PPPPP"]
                  : left > 0.05 ? [".....", ".....", ".PPP."]
                  : [".....", ".....", "....."]
        return scoop + ["nyyyn", ".nyn.", "..n.."]
    }
    static let food = [
        ["..G.", ".rr.", "rrrr", "rrrr", ".rr."],      // apple
        [".yy.", "ynyy", "yyny", ".yy."],              // cookie
        ["..i..", ".iii.", "iiiii", "ikkki"],          // rice ball
    ]
    static let flower = ["..P..", ".PyP.", "..P..", "..G..", ".GG..", "..G.."]
    static let cloud = ["...kkkk...", "..kWWWWk..", ".kWWWWWWkk", "kWWWWWWWWk", ".kkkkkkkk."]
    static let soil = ["....nnnnnnnnnnnn....", "..nnnNnnnnNnnnnNnn..", "nnnnnnnnnnnnnnnnnnnn"]
    static let butterflyA = ["P...P", "PPkPP", ".PkP."]
    static let butterflyB = ["..k..", "PPkPP", "P.k.P"]
    static let ball = ["rr", "rr"]

    // Typing activities
    static let pile = ["..nnnn..", ".nnNnnn.", "nnnnnNnn"]
    static let shovel = ["n....", ".n...", "..n..", "..ggg", "..gg."]
    /// Red-framed glasses on a half-size grid (2 pt cells), so each thin rim fits around an eye
    /// (eyes at sprite columns 9 and 12). Starts at half-column 10; the arm runs back toward the ear.
    static let glasses = [".......rrrr..rrrr", "rrrrrrrr..rrrr..r", ".......r..r..r..r",
                          ".......r..r..r..r", ".......r..r..r..r", ".......rrrr..rrrr"]

    // Mochi
    static let yarnA = [".PP.", "PkPP", "PPkP", ".PP."]
    static let yarnB = [".PP.", "PPkP", "PkPP", ".PP."]
    static let box = ["Uk..............kU", "kkkkkkkkkkkkkkkkkk", "kTTTTTTTUUTTTTTTTk", "kTTTTTTTUUTTTTTTTk",
                      "kTTTTTTTUUTTTTTTTk", "kTTTTTTTTTTTTTTTTk", "kkkkkkkkkkkkkkkkkk"]
    static let laserDot = ["r"]
    static let rod = ["......nn", ".....n.k", "....n..k", "...n...k", "..n....k", ".n.....k", "n......k", ".......r"]
    static let fish = [".ooo.o", "oooooo", ".ooo.o"]
}

let sparkleColors: [NSColor] = [.systemYellow, .systemPink, .systemOrange, .systemTeal, .systemPurple, .systemGreen]

func hueShifted(_ palette: [Character: NSColor], by amount: CGFloat, keys: [Character]) -> [Character: NSColor] {
    var out = palette
    for key in keys {
        guard let c = palette[key]?.usingColorSpace(.deviceRGB) else { continue }
        var h: CGFloat = 0, s: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        c.getHue(&h, saturation: &s, brightness: &b, alpha: &a)
        out[key] = NSColor(deviceHue: (h + amount).truncatingRemainder(dividingBy: 1), saturation: s, brightness: b, alpha: a)
    }
    return out
}

func lerp(_ a: CGFloat, _ b: CGFloat, _ t: Double) -> CGFloat { a + (b - a) * CGFloat(min(1, max(0, t))) }

extension PetView {
    /// Centre x of a sprite column, honouring which way the pet faces.
    func centerX(col: CGFloat) -> CGFloat {
        petX + (facingLeft ? 15.5 - col : col + 0.5) * pixel
    }

    /// Velocity range pointing the way the pet faces.
    func forward(_ a: CGFloat, _ b: CGFloat) -> ClosedRange<CGFloat> {
        facingLeft ? -b ... -a : a...b
    }

    /// True once per trick for each id.
    func once(_ id: Int) -> Bool { trickFlags.insert(id).inserted }

    /// True every `seconds` for each timer id.
    func every(_ id: Int, _ seconds: Double) -> Bool {
        if now - (trickTimers[id] ?? 0) >= seconds { trickTimers[id] = now; return true }
        return false
    }

    func startTrick(_ t: Trick) {
        trick = t
        lastTrick = t
        trickFlags = []
        trickTimers = [:]
        setMode(.trick, for: t.duration)
        trickStartFacing = facingLeft
        switch t {
        case .swim:
            setUpPool()
        case .laser:
            dotX = petCenterX
        case .twin:
            let roomRight = bounds.width - (petX + petSize), roomLeft = petX
            let side: CGFloat = roomRight >= roomLeft ? 1 : -1
            twinCenterX = min(max(petCenterX + side * 68, petSize / 2), bounds.width - petSize / 2)
        case .popFood:
            foodKind = Int.random(in: 0..<TrickArt.food.count)
        default:
            break
        }
    }

    func swimRange() -> ClosedRange<CGFloat> {
        let lo = max(0, poolLeft - 8), hi = max(lo, min(bounds.width - petSize, poolLeft + poolWidth - petSize + 8))
        return lo...hi
    }

    func butterflyPoint(_ t: Double, _ d: Double) -> NSPoint {
        let lo: CGFloat = 16, hi = max(lo + 1, bounds.width - 16)
        let mid = (lo + hi) / 2, half = max(0, (hi - lo) / 2 - 10)
        let x = mid + half * CGFloat(sin(t * 0.8)) + 8 * CGFloat(sin(t * 3.1))
        var y = 46 + 14 * CGFloat(sin(t * 2.2))
        if t > d - 1.5 { y += CGFloat(t - (d - 1.5)) * 70 }   // flies away at the end
        return NSPoint(x: x, y: y)
    }

    func firework(at p: NSPoint) {
        for i in 0..<16 {
            let a = Double(i) / 16 * 2 * .pi
            let speed = Double.random(in: 34...52)
            particles.append(Particle(text: i % 2 == 0 ? "✦" : "•", color: sparkleColors.randomElement()!, size: 11,
                                      x: p.x - 4, y: p.y, vx: CGFloat(cos(a) * speed), vy: CGFloat(sin(a) * speed),
                                      life: 1.3, maxLife: 1.3, ay: -24))
        }
    }

    // MARK: Trick motion and particles

    func stepTrick(_ tr: Trick, dt: Double) {
        let t = now - modeStart, d = modeUntil - modeStart
        switch tr {
        case .firework:
            let cycle = Int(t / 3.5), p = t.truncatingRemainder(dividingBy: 3.5)
            if p < 1.2, every(1, 0.08) {
                spawn("✦", .systemOrange, at: NSPoint(x: centerX(col: 17) - 4, y: 22), size: 9,
                      vx: -10...10, vy: 4...14, life: 0.45)
            }
            if p >= 2.0 && burstCycle != cycle {
                burstCycle = cycle
                firework(at: NSPoint(x: centerX(col: 17), y: 98))
            }

        case .flashlight:
            facingLeft = (Int(t / 2.7) % 2 == 1) != trickStartFacing
            if every(1, 2.2) { spawnAboveHead("?", .secondaryLabelColor, count: 1) }

        case .flipEgg:
            if t.truncatingRemainder(dividingBy: 2.0) < 0.7, every(1, 0.3) {
                spawn("~", .secondaryLabelColor, at: NSPoint(x: centerX(col: 19) - 4, y: 8 * pixel), size: 11,
                      vx: -3...3, vy: 10...16, life: 0.8)
            }

        case .campfire:
            if every(1, 0.45) {
                spawn("~", .secondaryLabelColor, at: NSPoint(x: centerX(col: 18) - 4, y: 6 * pixel), size: 12,
                      vx: -4...4, vy: 12...20, life: 1.6)
            }
            if every(2, 0.3) {
                spawn("·", .systemOrange, at: NSPoint(x: centerX(col: 18) - 3, y: 5 * pixel), size: 16,
                      vx: -10...10, vy: 18...32, life: 0.8)
            }

        case .hideInWater:
            if t > 0.8 && t < 3.5, every(1, 0.4) {
                spawn("°", .systemBlue, at: NSPoint(x: petCenterX + .random(in: -12...6), y: 6), size: 13,
                      vx: -3...3, vy: 14...22, life: 1.0)
            }
            if t > 5.2 && once(1) { facingLeft.toggle() }
            if t > 6.4 && once(2) { facingLeft.toggle() }
            if t > 7.9 && once(3) {
                for _ in 0..<10 {
                    spawn("•", .systemBlue, at: NSPoint(x: petCenterX - 4, y: 10), size: 10,
                          vx: -55...55, vy: 35...75, life: 0.9, ay: -170)
                }
            }

        case .spitWater:
            if t.truncatingRemainder(dividingBy: 0.9) < 0.1 && once(100 + Int(t / 0.9)) {
                for _ in 0..<7 {
                    spawn("•", .systemBlue, at: NSPoint(x: centerX(col: 15), y: 5.5 * pixel), size: 9,
                          vx: forward(55, 95), vy: 25...55, life: 1.1, ay: -170)
                }
            }

        case .swim:
            swimStep(dt: dt)

        case .bubbles:
            if every(1, 0.45) {
                spawn("○", .systemTeal, at: NSPoint(x: centerX(col: 15), y: 5.5 * pixel), size: .random(in: 9...15),
                      vx: forward(8, 22), vy: 12...24, life: 2.4)
            }

        case .iceCream:
            if every(1, 1.8) { spawnAboveHead("♥", .systemPink, count: 1) }

        case .twin:
            facingLeft = twinCenterX < petCenterX
            if t > 0.6 && once(1) { say("pop!", for: 1) }
            if t > 1.4, every(1, 1.1) {
                spawn("♪", .systemPink, at: NSPoint(x: (petCenterX + twinCenterX) / 2 - 4, y: petSize - 8), size: 13)
            }
            if t > d - 0.6 && once(2) { say("pop!", for: 1) }

        case .colorShift:
            if every(1, 0.22) {
                spawn("✦", sparkleColors.randomElement()!,
                      at: NSPoint(x: petCenterX + .random(in: -32...26), y: .random(in: 6...50)), size: 10,
                      vx: -4...4, vy: 4...10, life: 0.8)
            }

        case .popFood:
            if t > 1.2 && once(1) { say("!", for: 0.8) }
            if t > 2.0 && once(2) {
                say("a snack for you!", for: 3)
                spawn("✦", .systemYellow, at: NSPoint(x: centerX(col: 20) - 4, y: 12), size: 11, life: 0.9)
            }

        case .giveFlower:
            if once(1) { say("for you! 🌷", for: 4) }
            if t > 1, every(1, 1.1) { spawnAboveHead("♥", .systemPink, count: 1) }

        case .shower:
            let cloudLeft = petCenterX - 20
            if every(1, 0.06) {
                spawn("|", NSColor.systemBlue.withAlphaComponent(0.8),
                      at: NSPoint(x: cloudLeft + .random(in: 4...34), y: 17 * pixel - 6), size: 9,
                      vx: 0...0, vy: -95 ... -80, life: 0.65)
            }
            if every(2, 1.4) { spawn("♪", .systemPink, at: NSPoint(x: petCenterX + 22, y: 40), size: 13) }

        case .plantSelf:
            if t < 1 || (t > 7.5 && t < 8.3), every(1, 0.08) {
                spawn("·", .brown, at: NSPoint(x: petCenterX + .random(in: -26...20), y: 8), size: 16,
                      vx: -40...40, vy: 30...60, life: 0.7, ay: -150)
            }
            if t > 8.0 && once(1) { say("!", for: 1) }
            if t > 8.3 && t < 9.3 { facingLeft = Int(t / 0.2) % 2 == 0 }   // shakes off the dirt

        case .growUp:
            if (t > 0.5 && t < 3) || (t > 6 && t < 7), every(1, 0.15) {
                spawn("✦", .systemYellow, at: NSPoint(x: petCenterX + .random(in: -40...34), y: .random(in: 4...70)),
                      size: 10, vx: -4...4, vy: 4...12, life: 0.8)
            }
            if t > 3 && once(1) { say("so big!", for: 2) }

        case .chaseButterfly:
            let b = butterflyPoint(t, d)
            let behind: CGFloat = b.x > petCenterX ? 16 : -16
            let target = min(max(b.x - petSize / 2 - behind, 0), bounds.width - petSize)
            chaseMoving = !moveToward(target, speed: 55, dt: dt)
            if !chaseMoving { facingLeft = b.x < petCenterX }
            if t > d - 1.5 && once(1) { say("bye!", for: 1.5) }

        case .yarn:
            let ball = yarnX(t)
            let behind: CGFloat = ball > petCenterX ? 22 : -22
            let target = min(max(ball - petSize / 2 - behind, 0), bounds.width - petSize)
            chaseMoving = !moveToward(target, speed: 50, dt: dt)
            if !chaseMoving { facingLeft = ball < petCenterX }
            if t > 1, every(1, 2.4) { spawnAboveHead("♪", .systemPink, count: 1) }

        case .boxNap:
            if t > 1.4 && t < d - 1.4, every(1, 1.3) { spawnAboveHead("z", .secondaryLabelColor, count: 1, life: 2.2) }
            if t > d - 1.2 && once(1) { say("!", for: 1) }

        case .laser:
            // The dot jumps every 1.6 s; Mochi crouches, wiggles, then pounces on it.
            let cycle = Int(t / 1.6)
            if once(200 + cycle) && cycle > 0 {
                dotX = .random(in: 12...max(13, bounds.width - 12))
            }
            if t.truncatingRemainder(dividingBy: 1.6) > 0.55 {
                let target = min(max(dotX - petSize / 2 - (dotX > petCenterX ? 10 : -10), 0), bounds.width - petSize)
                chaseMoving = !moveToward(target, speed: 120, dt: dt)
            } else {
                chaseMoving = false
                facingLeft = dotX < petCenterX
            }

        case .fishing:
            let p = t.truncatingRemainder(dividingBy: 3.0)
            if p > 2.0 && once(300 + Int(t / 3.0)) {
                for _ in 0..<6 {
                    spawn("•", .systemBlue, at: NSPoint(x: centerX(col: 21) - 4, y: 12), size: 9,
                          vx: -30...30, vy: 30...55, life: 0.7, ay: -160)
                }
            }
            if p > 2.45 && once(400 + Int(t / 3.0)) { say("!", for: 0.6) }
        }
    }

    // MARK: Typing activities

    /// Starts the pet's own activity for while you type (laptop, cooking, swimming, reading, digging).
    func beginTyping(for seconds: Double = 3) {
        setMode(.typing, for: seconds)
        trickTimers = [:]
        trickFlags = []
        if pet.typing == .swimming { setUpPool() }
    }

    func setUpPool() {
        poolWidth = 30 * pixel
        poolLeft = min(max(petCenterX - poolWidth / 2, 0), max(0, bounds.width - poolWidth))
        targetX = swimRange().upperBound
    }

    func swimStep(dt: Double) {
        let range = swimRange()
        if moveToward(targetX, speed: 22, dt: dt) {
            targetX = abs(targetX - range.lowerBound) < 1 ? range.upperBound : range.lowerBound
        }
        if every(1, 0.35) {
            spawn("~", .systemBlue, at: NSPoint(x: petCenterX + (facingLeft ? 22 : -30), y: 6), size: 12,
                  vx: -4...4, vy: 2...6, life: 0.8)
        }
    }

    func stepTyping(dt: Double) {
        switch pet.typing {
        case .laptop, .readingGlasses:
            break
        case .cooking:
            if every(1, 0.35) {
                spawn("~", .secondaryLabelColor, at: NSPoint(x: centerX(col: 20) - 4, y: 8 * pixel), size: 11,
                      vx: -3...3, vy: 10...16, life: 0.8)
            }
        case .swimming:
            swimStep(dt: dt)
        case .digging:
            if Int(clock / 0.3) % 2 == 1, every(1, 0.3) {
                for _ in 0..<2 {
                    spawn("·", .brown, at: NSPoint(x: centerX(col: 18) - 3, y: 8), size: 16,
                          vx: forward(-10, 40), vy: 30...55, life: 0.7, ay: -150)
                }
            }
        }
    }

    /// A pet that doesn't crouch to sit (Puddle) holds the book lower, below its eyes.
    var bookLift: CGFloat { pet.sitBase == nil ? 3 : 1 }

    func glasses() -> FreeSprite {
        let half = pixel / 2, start: CGFloat = 10, width = CGFloat(TrickArt.glasses[0].count)
        let rows = facingLeft ? TrickArt.glasses.map { String($0.reversed()) } : TrickArt.glasses
        let x = petX + (facingLeft ? 32 - start - width : start) * half
        return FreeSprite(rows: rows, x: x, y: 11 * half, px: half)
    }

    func typingLook() -> Look {
        let t = now - modeStart
        let blink = clock.truncatingRemainder(dividingBy: 4.2) < 0.14
        let eyes: Eyes = blink ? .closed : .open
        func alt(_ period: Double) -> Bool { Int(clock / period) % 2 == 0 }
        var look = Look(frame: pet.frame(eyes, .sit), palette: pet.palette)
        switch pet.typing {
        case .laptop:
            look.props = currentProp().map { [$0] } ?? []
        case .cooking:
            // An egg in a pan over a little campfire, tossed now and then.
            let p = t.truncatingRemainder(dividingBy: 2.5)
            var egg = TrickArt.eggUp
            var lift: CGFloat = 7
            if p > 1.6 && p < 2.3 {
                lift = 7 + 6 * CGFloat(sin(.pi * (p - 1.6) / 0.7))
                egg = alt(0.1) ? TrickArt.eggUp : TrickArt.eggDown
            }
            look.frame = pet.frame(p > 2.3 ? .happy : eyes, .stand(0))
            look.props = [Prop(rows: alt(0.25) ? TrickArt.fireA : TrickArt.fireB, col: 17, lift: 0),
                          Prop(rows: TrickArt.pan, col: 13, lift: 5), Prop(rows: egg, col: 18, lift: lift)]
        case .swimming:
            look.yOffset = -3
            look.frame = pet.frame(eyes, .stand(Int(clock / 0.25) % 2))
            look.free = [FreeSprite(rows: TrickArt.pool(Int(poolWidth / pixel)), x: poolLeft, y: 0)]
        case .readingGlasses:
            look.props = [Prop(rows: Props.book(flip: t.truncatingRemainder(dividingBy: 3) < 0.25), col: 13, lift: bookLift)]
            look.free = [glasses()]
        case .digging:
            let down = !alt(0.3)
            let level = min(3, 1 + Int(t / 2))
            look.frame = pet.frame(eyes, .stand(down ? 1 : 0))
            look.props = [Prop(rows: Array(TrickArt.pile.suffix(level)), col: 15, lift: 0),
                          Prop(rows: TrickArt.shovel, col: 13, lift: down ? 1 : 3)]
        }
        return look
    }

    func yarnX(_ t: Double) -> CGFloat {
        let lo: CGFloat = 20, hi = max(lo + 1, bounds.width - 20)
        let mid = (lo + hi) / 2, half = max(0, (hi - lo) / 2 - 6)
        return mid + half * CGFloat(sin(t * 0.9 + 0.6))
    }

    // MARK: Trick looks

    func currentLook() -> Look {
        if mode == .trick, let trick, !dragging { return trickLook(trick) }
        if mode == .typing && !dragging { return typingLook() }
        var look = Look(frame: currentFrame(), palette: pet.palette, props: currentProp().map { [$0] } ?? [])
        if mode == .reading && pet.readsWithGlasses && !dragging { look.free.append(glasses()) }
        return look
    }

    private func trickLook(_ tr: Trick) -> Look {
        let t = now - modeStart, d = modeUntil - modeStart
        let blink = clock.truncatingRemainder(dividingBy: 4.2) < 0.14
        let eyes: Eyes = blink ? .closed : .open
        func alt(_ period: Double) -> Bool { Int(clock / period) % 2 == 0 }
        var look = Look(frame: pet.frame(eyes, .sit), palette: pet.palette)

        switch tr {
        case .firework:
            let p = t.truncatingRemainder(dividingBy: 3.5)
            look.frame = pet.frame(p > 2.0 ? .happy : eyes, .sit)
            if p < 1.2 { look.props = [Prop(rows: TrickArt.rocket, col: 16, lift: 0)] }
            else if p < 2.0 { look.props = [Prop(rows: TrickArt.rocket, col: 16, lift: CGFloat((p - 1.2) / 0.8) * 20)] }

        case .flashlight:
            look.frame = pet.frame(eyes, .stand(0))
            look.props = [Prop(rows: TrickArt.flashlight, col: 14, lift: 5)]
            look.beam = (NSPoint(x: centerX(col: 18) + (facingLeft ? -2 : 2), y: 6.5 * pixel),
                         CGFloat(0.05 + 0.22 * sin(t * 1.6)))

        case .flipEgg:
            let p = t.truncatingRemainder(dividingBy: 2.0)
            var egg = TrickArt.eggUp
            var lift: CGFloat = 7
            if p >= 0.7 && p < 1.6 {
                lift = 7 + 10 * CGFloat(sin(.pi * (p - 0.7) / 0.9))
                egg = alt(0.1) ? TrickArt.eggUp : TrickArt.eggDown
            }
            look.frame = pet.frame(p >= 1.6 ? .happy : eyes, .stand(0))
            look.props = [Prop(rows: TrickArt.pan, col: 13, lift: 5), Prop(rows: egg, col: 18, lift: lift)]

        case .campfire:
            look.frame = pet.frame(t > d - 2 ? .happy : eyes, .sit)
            look.props = [Prop(rows: alt(0.25) ? TrickArt.fireA : TrickArt.fireB, col: 15, lift: 0),
                          Prop(rows: TrickArt.stick(toasted: t > d * 0.5), col: 13, lift: 3)]

        case .hideInWater:
            // Sink, wait under water, peek out with just the eyes, then jump out with a splash.
            let y: Double
            switch t {
            case ..<0.8: y = -12 * t / 0.8
            case ..<3.5: y = -12
            case ..<4.3: y = -12 + 8 * (t - 3.5) / 0.8
            case ..<7.5: y = -4
            case ..<7.9: y = -4 + 7 * (t - 7.5) / 0.4
            case ..<8.2: y = 3 - 3 * (t - 7.9) / 0.3
            default: y = 0
            }
            look.yOffset = CGFloat(y)
            look.frame = pet.frame(t > 7.5 ? .happy : eyes, .stand(0))
            if t < 8.6 { look.props = [Prop(rows: TrickArt.pool(20), col: -2, lift: 0)] }

        case .spitWater:
            look.frame = pet.frame(t.truncatingRemainder(dividingBy: 0.9) < 0.2 ? .happy : .open, .sit)

        case .swim:
            look.yOffset = -3
            look.frame = pet.frame(.open, .stand(Int(clock / 0.25) % 2))
            look.free = [FreeSprite(rows: TrickArt.pool(Int(poolWidth / pixel)), x: poolLeft, y: 0)]

        case .bubbles:
            look.frame = pet.frame(alt(0.45) ? .happy : .open, .sit)

        case .iceCream:
            look.frame = pet.frame(alt(0.5) ? .happy : .open, .sit)
            look.props = [Prop(rows: TrickArt.iceCream(left: 1 - t / d), col: 14, lift: 3)]

        case .twin:
            let start = 0.6, end = d - 1.2
            if t < start {
                look.frame = pet.frame(.closed, .sit)
                break
            }
            let hop = sin((t - start) * 2 * .pi / 0.8)
            look.frame = pet.frame(.happy, .stand(0))
            look.yOffset = CGFloat(max(0, hop)) * 3
            var scale: CGFloat = 1
            var cx = twinCenterX
            if t < start + 0.5 {
                scale = 0.4 + 0.6 * CGFloat((t - start) / 0.5)
                cx = lerp(petCenterX, twinCenterX, (t - start) / 0.5)
            } else if t > end {
                let q = (t - end) / 0.6
                scale = 1 - 0.6 * CGFloat(min(1, q))
                cx = lerp(twinCenterX, petCenterX, q)
            }
            if t < end + 0.6 {
                look.twin = Twin(frame: pet.frame(.happy, .stand(0)), centerX: cx, scale: scale,
                                 yOffset: CGFloat(max(0, -hop)) * 3, facingLeft: twinCenterX > petCenterX)
            }
            if t > start + 0.9 && t < end - 0.2 {   // a ball passed back and forth
                let f = ((t - start - 0.9) / 1.2).truncatingRemainder(dividingBy: 2)
                let q = f < 1 ? f : 2 - f
                look.free = [FreeSprite(rows: TrickArt.ball, x: lerp(petCenterX, twinCenterX, q) - 4,
                                        y: 30 + 26 * CGFloat(sin(.pi * q)))]
            }

        case .colorShift:
            look.palette = hueShifted(pet.palette, by: CGFloat(t / d), keys: ["c", "d", "k"])
            look.frame = pet.frame(.happy, .stand(alt(0.5) ? 0 : 1))

        case .popFood:
            if t < 1.2 {
                look.frame = pet.frame(.closed, .stand(Int(clock / 0.12) % 2))
            } else {
                look.frame = pet.frame(.happy, .sit)
                let q = min(1, (t - 1.2) / 0.8)
                look.props = [Prop(rows: TrickArt.food[foodKind], col: 6 + 13 * CGFloat(q),
                                   lift: CGFloat((1 - q) * 10 + 12 * sin(.pi * q)))]
            }

        case .giveFlower:
            look.frame = pet.frame(.happy, .stand(0))
            look.props = [Prop(rows: TrickArt.flower, col: 14, lift: alt(0.6) ? 4 : 5)]

        case .shower:
            look.frame = pet.frame(.happy, alt(0.4) ? .stand(0) : .sit)
            look.free = [FreeSprite(rows: TrickArt.cloud, x: petCenterX - 20, y: 17 * pixel)]

        case .plantSelf:
            // Dig, sink until only the leaves show, nap, then pop out and shake off.
            let y: Double
            switch t {
            case ..<1: y = 0
            case ..<2.5: y = -8 * (t - 1) / 1.5
            case ..<7.5: y = -8
            case ..<8.0: y = -8 + 10 * (t - 7.5) / 0.5
            case ..<8.3: y = 2 - 2 * (t - 8.0) / 0.3
            default: y = 0
            }
            look.yOffset = CGFloat(y)
            look.frame = pet.frame(t > 2.5 && t < 7.5 ? .closed : .open, t < 1 ? .stand(Int(clock / 0.15) % 2) : .stand(0))
            if t < 9.2 { look.props = [Prop(rows: TrickArt.soil, col: -2, lift: 0)] }

        case .growUp:
            let scale: CGFloat
            switch t {
            case ..<1: scale = 1
            case ..<3: scale = 1 + 0.125 * CGFloat(min(4, Int((t - 1) / 0.5) + 1))
            case ..<6: scale = 1.5
            case ..<7: scale = 1.5 - 0.125 * CGFloat(min(4, Int((t - 6) / 0.25) + 1))
            default: scale = 1
            }
            look.scale = scale
            look.frame = pet.frame(scale > 1 ? .happy : eyes, .stand(0))

        case .chaseButterfly:
            look.frame = pet.frame(.happy, chaseMoving ? .stand(Int(clock / 0.14) % 2) : .stand(0))
            let b = butterflyPoint(t, d)
            look.free = [FreeSprite(rows: alt(0.15) ? TrickArt.butterflyA : TrickArt.butterflyB,
                                    x: b.x - 7.5, y: b.y, px: 3)]

        case .yarn:
            look.frame = pet.frame(.happy, chaseMoving ? .stand(Int(clock / 0.14) % 2) : .stand(0))
            let x = yarnX(t)
            let rolling = Int(x / 6) % 2 == 0
            look.free = [FreeSprite(rows: rolling ? TrickArt.yarnA : TrickArt.yarnB, x: x - 8, y: 0)]

        case .boxNap:
            // Hop into the box, nap with just the head showing, then pop up.
            let y: Double = t < 0.8 ? 2 + 8 * sin(.pi * t / 0.8) : (t > d - 1.2 ? 3 : 2)
            look.yOffset = CGFloat(y)
            look.frame = pet.frame(t > 1.2 && t < d - 1.2 ? .closed : .happy, .sit)
            look.props = [Prop(rows: TrickArt.box, col: -1, lift: 0)]

        case .laser:
            let p = t.truncatingRemainder(dividingBy: 1.6)
            if chaseMoving {
                look.frame = pet.frame(.open, .stand(0))
                look.yOffset = CGFloat(4 * sin(.pi * min(1, (p - 0.55) / 0.5)))
            } else {
                look.frame = pet.frame(.open, alt(0.12) ? .sit : .stand(0))   // wiggles before pouncing
            }
            look.free = [FreeSprite(rows: TrickArt.laserDot, x: dotX - 2.5, y: 0, px: 5)]

        case .fishing:
            let p = t.truncatingRemainder(dividingBy: 3.0)
            look.frame = pet.frame(p > 2.3 ? .happy : eyes, .sit)
            look.props = [Prop(rows: TrickArt.pool(10), col: 15, lift: 0), Prop(rows: TrickArt.rod, col: 13, lift: 3)]
            if p > 2.0 && p < 2.9 {
                let q = (p - 2.0) / 0.9
                look.props.append(Prop(rows: TrickArt.fish, col: 18, lift: 3 + 9 * CGFloat(sin(.pi * q))))
            }
        }
        return look
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

    var demoQueue: [(seconds: Double, run: () -> Void)] = []
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

        // `--demo` cycles through every activity, reminder and trick; `--tricks` plays only the pet's tricks.
        let args = CommandLine.arguments
        if args.contains("--demo") || args.contains("--tricks") || args.contains("--typing") {
            let v = view!
            if args.contains("--demo") {
                demoQueue = [
                    (7, { v.beginTyping(for: 6) }),
                    (7, { v.setMode(.reading, for: 6) }),
                    (7, { v.remind(.water) }),
                    (7, { v.remind(.lunch) }),
                    (7, { v.remind(.stretch) }),
                ]
            }
            if args.contains("--demo") && v.pet.sweeps {
                demoQueue.insert((7, { v.setMode(.sweeping, for: 6); v.targetX = v.randomTarget(near: v.petX, within: 50) }), at: 1)
            }
            if args.contains("--typing") { demoQueue = [(9, { v.beginTyping(for: 8) })] }
            for trick in v.pet.tricks where !args.contains("--typing") {
                demoQueue.append((trick.duration + 0.5, { v.startTrick(trick) }))
            }
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
            let next = demoQueue.removeFirst()
            next.run()
            nextDemo = now + next.seconds
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

        if !view.pet.tricks.isEmpty {
            let trickItem = NSMenuItem(title: "Tricks", action: nil, keyEquivalent: "")
            let trickMenu = NSMenu()
            let random = NSMenuItem(title: "Surprise Me", action: #selector(randomTrick), keyEquivalent: "")
            random.target = self
            trickMenu.addItem(random)
            trickMenu.addItem(.separator())
            for trick in view.pet.tricks {
                let item = NSMenuItem(title: trick.title, action: #selector(pickTrick(_:)), keyEquivalent: "")
                item.tag = Trick.allCases.firstIndex(of: trick)!
                item.target = self
                trickMenu.addItem(item)
            }
            trickItem.submenu = trickMenu
            menu.addItem(trickItem)
        }
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

    @objc func pickTrick(_ sender: NSMenuItem) {
        view.bubbleText = nil
        view.startTrick(Trick.allCases[sender.tag])
    }

    @objc func randomTrick() {
        view.bubbleText = nil
        if let trick = view.pet.tricks.randomElement() { view.startTrick(trick) }
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
