// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "NucleusSwift",
    platforms: [
        .iOS(.v16),
    ],
    products: [
        .library(
            name: "NucleusSwift",
            targets: ["NucleusSwift"]
        ),
    ],
    targets: [
        .target(
            name: "NucleusSwift",
            path: "Sources/NucleusSwift"
        ),
        .testTarget(
            name: "NucleusSwiftTests",
            dependencies: ["NucleusSwift"]
        ),
    ]
)
