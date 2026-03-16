class Xurl < Formula
  desc "Fast, ergonomic CLI for the X (Twitter) API"
  homepage "https://github.com/brettdavies/xurl-rs"
  url "https://github.com/brettdavies/xurl-rs/archive/refs/tags/v0.1.0.tar.gz"
  license "MIT"
  head "https://github.com/brettdavies/xurl-rs.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    # Install shell completions
    generate_completions_from_executable(bin/"xurl", "--generate-completion")
  end

  test do
    assert_match "xurl", shell_output("#{bin}/xurl --help")
  end
end
