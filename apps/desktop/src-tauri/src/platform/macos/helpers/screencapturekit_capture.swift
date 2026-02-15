import AppKit
import Foundation
import ScreenCaptureKit

@main
struct CaptureMain {
    static func main() async {
        guard CommandLine.arguments.count >= 2 else {
            fputs("missing output path\n", stderr)
            exit(2)
        }

        let outputPath = CommandLine.arguments[1]

        if #available(macOS 14.0, *) {
            do {
                let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
                guard let display = content.displays.first else {
                    fputs("no display available\n", stderr)
                    exit(3)
                }
                let filter = SCContentFilter(display: display, excludingWindows: [])
                let cfg = SCStreamConfiguration()
                cfg.width = Int(display.width)
                cfg.height = Int(display.height)
                let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: cfg)
                let rep = NSBitmapImageRep(cgImage: image)
                guard let data = rep.representation(using: .png, properties: [:]) else {
                    fputs("failed to encode png\n", stderr)
                    exit(4)
                }
                try data.write(to: URL(fileURLWithPath: outputPath))
                print("ok")
                return
            } catch {
                fputs("ScreenCaptureKit error: \(error)\n", stderr)
                exit(5)
            }
        }

        fputs("ScreenCaptureKit requires macOS 14+\n", stderr)
        exit(6)
    }
}
