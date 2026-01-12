class Piper < Formula
  desc "The Middle-Out Data Optimizer"
  homepage "https://github.com/biplavbarua/piper"
  url "https://github.com/biplavbarua/piper/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "0f63fe028447ed5b86901ed49d3d0a78e3b07bc9986c79301fad560960e51dcd"
  license "MIT"
  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end
end
