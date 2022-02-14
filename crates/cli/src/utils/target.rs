use std::process::Command;

use phf::phf_map;
use serde::{ser::SerializeMap, Serialize, Serializer};

pub const AVAILABLE_TARGETS: &[&str] = &[
  "aarch64-apple-darwin",
  "aarch64-linux-android",
  "aarch64-unknown-linux-gnu",
  "aarch64-unknown-linux-musl",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-unknown-freebsd",
  "i686-pc-windows-msvc",
  "armv7-unknown-linux-gnueabihf",
  "armv7-linux-androideabi",
];

pub const DEFAULT_TARGETS: &[&str] = &[
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeArch {
  x32,
  x64,
  ia32,
  arm,
  arm64,
  mips,
  mipsel,
  ppc,
  ppc64,
  s390,
  s390x,
}

impl NodeArch {
  fn from_str(s: &str) -> Option<Self> {
    match s {
      "x32" => Some(NodeArch::x32),
      "x86_64" => Some(NodeArch::x64),
      "i686" => Some(NodeArch::ia32),
      "armv7" => Some(NodeArch::arm),
      "aarch64" => Some(NodeArch::arm64),
      "mips" => Some(NodeArch::mips),
      "mipsel" => Some(NodeArch::mipsel),
      "ppc" => Some(NodeArch::ppc),
      "ppc64" => Some(NodeArch::ppc64),
      "s390" => Some(NodeArch::s390),
      "s390x" => Some(NodeArch::s390x),
      _ => None,
    }
  }
}

impl std::fmt::Display for NodeArch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodeArch::x32 => write!(f, "x86"),
      NodeArch::x64 => write!(f, "x64"),
      NodeArch::ia32 => write!(f, "ia32"),
      NodeArch::arm => write!(f, "arm"),
      NodeArch::arm64 => write!(f, "arm64"),
      NodeArch::mips => write!(f, "mips"),
      NodeArch::mipsel => write!(f, "mipsel"),
      NodeArch::ppc => write!(f, "ppc"),
      NodeArch::ppc64 => write!(f, "ppc64"),
      NodeArch::s390 => write!(f, "s390"),
      NodeArch::s390x => write!(f, "s390x"),
    }
  }
}

impl Serialize for NodeArch {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&format!("{}", self))
  }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodePlatform {
  Darwin,
  Freebsd,
  Windows,
  Linux,
  Android,
  Fuchsia,
  Unknown(String),
}

impl NodePlatform {
  fn from_str(s: &str) -> Self {
    match s {
      "darwin" => NodePlatform::Darwin,
      "freebsd" => NodePlatform::Freebsd,
      "windows" => NodePlatform::Windows,
      "linux" => NodePlatform::Linux,
      "android" => NodePlatform::Android,
      "fuchsia" => NodePlatform::Fuchsia,
      _ => NodePlatform::Unknown(s.to_owned()),
    }
  }
}

impl std::fmt::Display for NodePlatform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodePlatform::Darwin => write!(f, "darwin"),
      NodePlatform::Freebsd => write!(f, "freebsd"),
      NodePlatform::Windows => write!(f, "win32"),
      NodePlatform::Linux => write!(f, "linux"),
      NodePlatform::Android => write!(f, "android"),
      NodePlatform::Fuchsia => write!(f, "fuchsia"),
      NodePlatform::Unknown(s) => write!(f, "{}", s),
    }
  }
}

impl Serialize for NodePlatform {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&format!("{}", self))
  }
}

pub fn get_system_default_target() -> String {
  let output = Command::new("rustc")
    .arg("-vV")
    .output()
    .expect("Failed to get rustc version information");

  unsafe {
    let output = String::from_utf8_unchecked(output.stdout);

    output
      .lines()
      .find(|line| line.starts_with("host:"))
      .map(|line| line.split(' ').nth(1))
      .expect("Failed to get rustc version information")
      .unwrap()
      .to_owned()
  }
}

#[derive(Clone, Debug)]
pub struct GithubWorkflowConfig {
  pub host: &'static str,
  pub docker_image: Option<&'static str>,
  pub setup: Option<&'static str>,
}

impl Default for GithubWorkflowConfig {
  fn default() -> Self {
    Self {
      host: "ubuntu-latest",
      docker_image: None,
      setup: None,
    }
  }
}

impl Serialize for GithubWorkflowConfig {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(2))?;
    map.serialize_entry("host", self.host)?;
    if let Some(docker_image) = &self.docker_image {
      map.serialize_entry("docker_image", docker_image)?;
    }
    if let Some(setup) = &self.setup {
      let scripts = setup
        .lines()
        .filter_map(|line| {
          let line = line.trim();
          if line.is_empty() {
            None
          } else {
            Some(line)
          }
        })
        .collect::<Vec<_>>();
      map.serialize_entry("setup", &scripts)?;
    }
    map.end()
  }
}

static TARGET_CONFIG_MAP: phf::Map<&'static str, GithubWorkflowConfig> = phf_map! {
  "x86_64-apple-darwin" => GithubWorkflowConfig {
    host: "macos-latest",
    docker_image: None,
    setup: None,
  },
  "x86_64-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: None,
  },
  "i686-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: None,
  },
  "x86_64-unknown-linux-gnu" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian"),
    setup: None,
  },
  "x86_64-unknown-linux-musl" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine"),
    setup: None,
  },
  // CHECK
  "x86_64-unknown-freebsd" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: None,
  },
  "aarch64-apple-darwin" => GithubWorkflowConfig {
    host: "macos-latest",
    docker_image: None,
    setup: Some(r#"
      sudo rm -Rf /Library/Developer/CommandLineTools/SDKs/*
      export CC=$(xcrun -f clang)
      export CXX=$(xcrun -f clang++)
      export SDK_ROOT=$(xcrun --sdk macosx --show-sdk-path)
      export CFLAGS="-isysroot $SDK_ROOT -isystem $SDK_ROOT"
    "#),
  },
  "aarch64-unknown-linux-gnu" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian-aarch64"),
    setup: Some(r"
      sudo apt-get update
      sudo apt-get install g++-aarch64-linux-gnu gcc-aarch64-linux-gnu -y
    "),
  },
  "aarch64-unknown-linux-musl" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine"),
    setup: None,
  },
  "aarch64-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: Some("rustup target add aarch64-unknown-linux-musl"),
  },
  "armv7-unknown-linux-gnueabihf" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: Some(r"
      sudo apt-get update
      sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf -y
    "),
  },
  "aarch64-linux-android" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: Some(r#"
      export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang"
      export CC="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang"
      export CXX="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang++"
      export PATH="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin:\${PATH}"
    "#),
  },
  "armv7-linux-androideabi" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: Some(r#"
      export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang"
      export CC="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang"
      export CXX="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi24-clang++"
      export PATH="\${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin:\${PATH}"
    "#),
  },
};

pub fn get_github_workflow_config(target: &str) -> GithubWorkflowConfig {
  TARGET_CONFIG_MAP.get(target).cloned().unwrap_or_default()
}

#[derive(Clone, Debug, Serialize)]
pub struct Target {
  pub triple: String,
  pub platform_arch_abi: String,
  pub platform: NodePlatform,
  pub arch: NodeArch,
  pub abi: Option<String>,
}

impl Target {
  pub fn new(triple: &str) -> Self {
    let mut target = triple.to_string();
    // armv7-linux-androideabi => armv7-linux-android-eabi
    if target.ends_with("androideabi") {
      target.insert(target.len() - 4, '-');
    }

    let parts = target.split('-').collect::<Vec<_>>();
    let (cpu, sys, abi) = if parts.len() == 2 {
      // aarch64-fuchsia
      // ^ cpu   ^ sys
      (parts[0], parts[1], None)
    } else {
      // aarch64-unknown-linux-musl
      // ^ cpu           ^ sys ^ abi
      // aarch64-apple-darwin
      // ^ cpu         ^ sys  (abi is None)
      (parts[0], parts[2], parts.get(3))
    };

    let platform = NodePlatform::from_str(sys);
    let arch = NodeArch::from_str(cpu).unwrap_or_else(|| panic!("unsupported cpu arch {}", cpu));

    Self {
      triple: triple.to_owned(),
      platform_arch_abi: if abi.is_some() {
        format!("{}-{}-{}", platform, arch, abi.unwrap())
      } else {
        format!("{}-{}", platform, arch)
      },
      platform,
      arch,
      abi: abi.map(|s| s.to_string()),
    }
  }
}

impl PartialEq for Target {
  fn eq(&self, other: &Self) -> bool {
    self.triple == other.triple
  }
}

impl<T> From<T> for Target
where
  T: AsRef<str>,
{
  fn from(s: T) -> Self {
    Self::new(s.as_ref())
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_get_system_default_target() {
    let target = get_system_default_target();
    assert!(!target.is_empty());
    let target = Target::from(&target);
    if cfg!(target_os = "windows") {
      assert_eq!(target.platform, NodePlatform::Windows);
    } else if cfg!(target_os = "macos") {
      assert_eq!(target.platform, NodePlatform::Darwin);
    } else if cfg!(target_os = "linux") {
      assert_eq!(target.platform, NodePlatform::Linux);
    }
  }

  #[test]
  fn test_target_from_str() {
    // crate will be built both for lib and binary
    // only need snapshot test once.
    use insta::assert_debug_snapshot;
    let targets = AVAILABLE_TARGETS
      .iter()
      .map(Target::from)
      .collect::<Vec<_>>();

    assert_debug_snapshot!(&targets);
  }
}
