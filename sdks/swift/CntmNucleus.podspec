Pod::Spec.new do |s|
  s.name         = 'CntmNucleus'
  s.version      = '0.2.0'
  s.summary      = 'Nucleus authentication SDK for iOS/macOS.'
  s.homepage     = 'https://github.com/cntm-labs/nucleus'
  s.license      = { :type => 'MIT', :file => 'LICENSE' }
  s.author       = { 'cntm-labs' => 'dev@cntm-labs.dev' }
  s.source       = { :git => 'https://github.com/cntm-labs/nucleus.git', :tag => "CntmNucleus-v#{s.version}" }
  s.ios.deployment_target = '16.0'
  s.osx.deployment_target = '13.0'
  s.swift_version = '5.9'
  s.source_files = 'Sources/NucleusSwift/**/*.swift'
  s.frameworks = 'Foundation', 'Security'
  s.weak_frameworks = 'SwiftUI', 'Combine', 'AuthenticationServices'
end
