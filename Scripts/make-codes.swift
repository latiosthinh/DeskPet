// Makes DeskPet unlock codes.
//
//   swift Scripts/make-codes.swift 5        # print 5 new codes
//   swift Scripts/make-codes.swift --public # print the public key to put in main.swift
//
// The first run creates a private signing key at ~/.config/deskpet/signing-key.
// Keep that file safe and private: anyone with it can make codes, and if you
// lose it you can't make codes that existing copies of the app accept.
// Codes are also appended to ~/.config/deskpet/codes.txt so you have a record.

import CryptoKit
import Foundation

let dir = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".config/deskpet")
let keyFile = dir.appendingPathComponent("signing-key")
let logFile = dir.appendingPathComponent("codes.txt")

func loadOrCreateKey() throws -> Curve25519.Signing.PrivateKey {
    if let data = try? Data(contentsOf: keyFile) {
        return try Curve25519.Signing.PrivateKey(rawRepresentation: data)
    }
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true,
                                            attributes: [.posixPermissions: 0o700])
    let key = Curve25519.Signing.PrivateKey()
    try key.rawRepresentation.write(to: keyFile)
    try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: keyFile.path)
    FileHandle.standardError.write("Created a new signing key at \(keyFile.path). Back it up somewhere safe.\n".data(using: .utf8)!)
    return key
}

/// Crockford base32: no I, L, O or U, so codes are easy to read back.
let alphabet = Array("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

func base32(_ data: Data) -> String {
    var out = "", buffer = 0, bits = 0
    for byte in data {
        buffer = (buffer << 8) | Int(byte)
        bits += 8
        while bits >= 5 {
            out.append(alphabet[(buffer >> (bits - 5)) & 31])
            bits -= 5
        }
    }
    if bits > 0 { out.append(alphabet[(buffer << (5 - bits)) & 31]) }
    return out
}

/// Payload: version (1), what it unlocks (1: all pets), and 6 random bytes as a serial.
func makeCode(_ key: Curve25519.Signing.PrivateKey) throws -> String {
    var payload = Data([1, 1])
    payload.append(contentsOf: (0..<6).map { _ in UInt8.random(in: 0...255) })
    let signature = try key.signature(for: payload)
    let body = base32(payload + signature)
    let groups = stride(from: 0, to: body.count, by: 6).map { i -> String in
        let start = body.index(body.startIndex, offsetBy: i)
        let end = body.index(start, offsetBy: min(6, body.count - i))
        return String(body[start..<end])
    }
    return "PET-" + groups.joined(separator: "-")
}

do {
    let key = try loadOrCreateKey()
    let args = CommandLine.arguments.dropFirst()
    if args.first == "--public" {
        print(key.publicKey.rawRepresentation.base64EncodedString())
        exit(0)
    }
    let count = Int(args.first ?? "1") ?? 1
    var lines: [String] = []
    for _ in 0..<count { lines.append(try makeCode(key)) }
    lines.forEach { print($0) }
    let stamp = ISO8601DateFormatter().string(from: Date())
    let record = lines.map { "\(stamp)  \($0)\n" }.joined()
    if let handle = try? FileHandle(forWritingTo: logFile) {
        handle.seekToEndOfFile()
        handle.write(record.data(using: .utf8)!)
        handle.closeFile()
    } else {
        try record.write(to: logFile, atomically: true, encoding: .utf8)
    }
} catch {
    FileHandle.standardError.write("Couldn't make codes: \(error)\n".data(using: .utf8)!)
    exit(1)
}
