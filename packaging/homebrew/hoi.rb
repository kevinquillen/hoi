# Homebrew formula template for Hoi.

class Hoi < Formula
  desc "Cross-platform command runner for development teams"
  homepage "https://github.com/kevinquillen/hoi"
  version "0.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-macOS-arm64.tar.gz"
      sha256 "REPLACE_WITH_MACOS_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-macOS-x86_64.tar.gz"
      sha256 "REPLACE_WITH_MACOS_X86_64_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-Linux-musl-arm64.tar.gz"
      sha256 "REPLACE_WITH_LINUX_ARM64_SHA256"
    end
    on_intel do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-Linux-musl-x86_64.tar.gz"
      sha256 "REPLACE_WITH_LINUX_X86_64_SHA256"
    end
  end

  def install
    bin.install "hoi"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/hoi --version")
  end
end
