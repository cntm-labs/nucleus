// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "CntmNucleus",
    platforms: [
        .iOS(.v16),
        .macOS(.v13),
    ],
    products: [
        .library(
            name: "CntmNucleus",
            targets: ["CntmNucleus"]
        ),
    ],
    targets: [
        .target(
            name: "CntmNucleus",
            path: "Sources/CntmNucleus"
        ),
        .testTarget(
            name: "CntmNucleusTests",
            dependencies: ["CntmNucleus"]
        ),
    ]
)
