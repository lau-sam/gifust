class Gifust < Formula
  desc "Convert videos to GIFs from the CLI or an interactive TUI"
  homepage "https://github.com/lau-sam/gifust"
  url "https://github.com/lau-sam/gifust/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_TARBALL_SHA256"
  license "MIT"
  head "https://github.com/lau-sam/gifust.git", branch: "main"

  depends_on "rust" => :build
  depends_on "ffmpeg"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "gifust", shell_output("#{bin}/gifust --version")
  end
end
