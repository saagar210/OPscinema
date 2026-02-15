import AppKit
import Foundation
import Vision

struct VisionBlock: Codable {
    let text: String
    let confidence: Double
    let x: Double
    let y: Double
    let w: Double
    let h: Double
}

let cgImage: CGImage = {
    if CommandLine.arguments.count >= 2 && CommandLine.arguments[1] == "--stdin-image" {
        let data = FileHandle.standardInput.readDataToEndOfFile()
        guard let image = NSImage(data: data),
              let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil)
        else {
            fputs("failed to load stdin image\n", stderr)
            exit(3)
        }
        return cgImage
    }

    guard CommandLine.arguments.count >= 2 else {
        fputs("missing image path\n", stderr)
        exit(2)
    }
    let imagePath = CommandLine.arguments[1]
    let url = URL(fileURLWithPath: imagePath)
    guard let image = NSImage(contentsOf: url),
          let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil)
    else {
        fputs("failed to load image\n", stderr)
        exit(3)
    }
    return cgImage
}()

let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true

let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
do {
    try handler.perform([request])
} catch {
    fputs("vision perform error: \(error)\n", stderr)
    exit(4)
}

let results = (request.results ?? []).compactMap { observation -> VisionBlock? in
    guard let top = observation.topCandidates(1).first else { return nil }
    let bb = observation.boundingBox
    return VisionBlock(
        text: top.string,
        confidence: Double(top.confidence),
        x: Double(bb.minX),
        y: Double(bb.minY),
        w: Double(bb.width),
        h: Double(bb.height)
    )
}

do {
    let data = try JSONEncoder().encode(results)
    FileHandle.standardOutput.write(data)
} catch {
    fputs("encode error: \(error)\n", stderr)
    exit(5)
}
