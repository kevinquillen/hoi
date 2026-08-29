class Hoi < Formula
  desc "Cross-platform command runner for development teams"
  homepage "https://github.com/kevinquillen/hoi"
  version "0.7.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-macOS-arm64.tar.gz"
      sha256 "9b2fe5354e16990ab2a432480de34cc03ebe81c05bccacee44adfdef2de84083"
    end
    on_intel do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-macOS-x86_64.tar.gz"
      sha256 "6b2cda61fa376f4bfdab61fa66dd4d6b0284e0f04abc29f7b8343cc6bb36e495"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-Linux-musl-arm64.tar.gz"
      sha256 "5124a84f3820a002f8a06bf6b35205b34c5c92b83fb927e42bf06102ee6820d8"
    end
    on_intel do
      url "https://github.com/kevinquillen/hoi/releases/download/v#{version}/hoi-Linux-musl-x86_64.tar.gz"
      sha256 "189398124fc77b775093ab62f82ee4a41da0e983e2910daef106fb5b56000cf0"
    end
  end

  def install
    bin.install "hoi"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/hoi --version")
  end
end
