// Draws the placeholder app icon (assets/AppIcon.png, 1024x1024). Replace that
// PNG with your own artwork; this script only exists to make the placeholder.
// Run: swift scripts/generate-icon.swift
import AppKit

let size = 1024
let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
let output = root.appendingPathComponent("assets/AppIcon.png")
let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
// macOS icon grid: an 824pt rounded square centred in 1024.
let tile = NSRect(x: 100, y: 100, width: 824, height: 824)
let shape = NSBezierPath(roundedRect: tile, xRadius: 185, yRadius: 185)
NSGradient(starting: NSColor(calibratedRed: 0.12, green: 0.14, blue: 0.16, alpha: 1),
           ending: NSColor(calibratedRed: 0.05, green: 0.06, blue: 0.07, alpha: 1))!
    .draw(in: shape, angle: -90)
// Three stacked "window" bars in the accent colour.
let accent = NSColor(calibratedRed: 0.91, green: 0.84, blue: 0.72, alpha: 1)
for (index, width) in [520.0, 420.0, 300.0].enumerated() {
    let bar = NSRect(x: 252, y: 600 - Double(index) * 150, width: width, height: 96)
    accent.withAlphaComponent(1 - Double(index) * 0.25).setFill()
    NSBezierPath(roundedRect: bar, xRadius: 48, yRadius: 48).fill()
}
NSGraphicsContext.restoreGraphicsState()
try bitmap.representation(using: .png, properties: [:])!.write(to: output)
print("Wrote \(output.path)")
