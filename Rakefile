# frozen_string_literal: true

require 'open-uri'
require 'shellwords'

require 'bundler/audit/task'
require 'rubocop/rake_task'
require 'tomlrb'

task default: %i[format lint]

desc 'Lint sources'
task lint: %i[lint:clippy lint:rubocop:autocorrect]

namespace :lint do
  RuboCop::RakeTask.new(:rubocop)

  desc 'Lint Rust sources with Clippy'
  task :clippy do
    sh 'cargo clippy --workspace --all-features --all-targets'
    Dir.chdir('spec-runner') do
      sh 'cargo clippy --workspace --all-features --all-targets'
    end
    Dir.chdir('ui-tests') do
      sh 'cargo clippy --workspace --all-features --all-targets'
    end
  end

  desc 'Lint Rust sources with Clippy restriction pass (unenforced lints)'
  task :'clippy:restriction' do
    lints = [
      'clippy::dbg_macro',
      'clippy::get_unwrap',
      'clippy::indexing_slicing',
      'clippy::panic',
      'clippy::print_stdout',
      'clippy::expect_used',
      'clippy::unwrap_used',
      'clippy::todo',
      'clippy::unimplemented',
      'clippy::unreachable'
    ]
    command = ['cargo', 'clippy', '--'] + lints.flat_map { |lint| ['-W', lint] }
    sh command.shelljoin
  end
end

desc 'Format sources'
task format: %i[format:rust format:text format:c]

namespace :format do
  desc 'Format Rust sources with rustfmt'
  task :rust do
    sh 'rustup run --install nightly cargo fmt -- --color=auto'
    Dir.chdir('spec-runner') do
      sh 'rustup run --install nightly cargo fmt -- --color=auto'
    end
    Dir.chdir('ui-tests') do
      sh 'rustup run --install nightly cargo fmt -- --color=auto'
    end
  end

  desc 'Format text, YAML, and Markdown sources with prettier'
  task :text do
    sh 'npm run fmt'
  end

  desc 'Format .c and .h sources with clang-format'
  task :c do
    sh 'npm run fmt:c'
  end
end

desc 'Format sources'
task fmt: %i[fmt:rust fmt:text fmt:c]

namespace :fmt do
  desc 'Format Rust sources with rustfmt'
  task :rust do
    sh 'rustup run --install nightly cargo fmt -- --color=auto'
    Dir.chdir('spec-runner') do
      sh 'rustup run --install nightly cargo fmt -- --color=auto'
    end
    Dir.chdir('ui-tests') do
      sh 'rustup run --install nightly cargo fmt -- --color=auto'
    end
  end

  desc 'Format text, YAML, and Markdown sources with prettier'
  task :text do
    sh 'npm run fmt'
  end

  desc 'Format .c and .h sources with clang-format'
  task :c do
    sh 'npm run fmt:c'
  end
end

desc 'Build Rust workspace'
task :build do
  sh 'cargo build --workspace'
end

desc 'Build Rust workspace and sub-workspaces'
task :'build:all' do
  sh 'cargo build --workspace'
  Dir.chdir('fuzz') do
    sh 'cargo build --workspace'
  end
  Dir.chdir('spec-runner') do
    sh 'cargo build --workspace'
  end
  Dir.chdir('ui-tests') do
    sh 'cargo build --workspace'
  end
end

desc 'Generate Rust API documentation'
task :doc do
  ENV['RUSTDOCFLAGS'] = '-D warnings -D rustdoc::broken_intra_doc_links --cfg docsrs'
  sh 'rustup run --install nightly cargo doc --workspace'
end

desc 'Generate Rust API documentation and open it in a web browser'
task :'doc:open' do
  ENV['RUSTDOCFLAGS'] = '-D warnings -D rustdoc::broken_intra_doc_links --cfg docsrs'
  sh 'rustup run --install nightly cargo doc --workspace --open'
end

desc 'Run enforced ruby/spec suite'
task :spec do
  Dir.chdir('spec-runner') do
    sh 'cargo run -q -- enforced-specs.toml'
  end
end

desc 'Run Artichoke unit tests'
task test: %i[test:unit]

namespace :test do
  # TODO: Add fuzz into all list when tests work
  desc 'Run all tests'
  task all: %i[unit ui]

  desc 'Run fuzz tests (Fuzz the interpreter for crashes with arbitrary input)'
  task :fuzz do
    Dir.chdir('fuzz') do
      sh 'cargo test --workspace'
    end
  end

  desc 'Run ui tests (check exact stdout/stderr of Artichoke binaries)'
  task :ui do
    sh 'cargo build'
    Dir.chdir('ui-tests') do
      sh 'cargo test --workspace'
    end
  end

  desc 'Run unit tests'
  task :unit do
    sh 'cargo test --workspace'
  end
end

desc 'Run Artichoke with LeakSanitizer'
task :'sanitizer:leak' do
  ENV['RUSTFLAGS'] = '-Z sanitizer=leak'
  ENV['RUST_BACKTRACE'] = '1'
  host = `rustc -vV | grep host | cut -d' ' -f2`.chomp
  command = ['rustup', 'run', '--install', 'nightly', 'cargo', 'test', '--workspace', '--all-features', '--target', host]
  sh command.shelljoin
end

Bundler::Audit::Task.new

namespace :toolchain do
  desc 'Sync Rust toolchain to all sources'
  task sync: %i[sync:manifests sync:ci]

  rust_toolchain = Tomlrb.load_file('rust-toolchain.toml', symbolize_keys: true)
  toolchain_version = rust_toolchain[:toolchain][:channel]

  namespace :sync do
    desc 'Sync the root rust-toolchain version to all crate manifests'
    task :manifests do
      regexp = /^rust-version = "(.*)"$/
      next_rust_version = "rust-version = \"#{toolchain_version}\""

      pkg_files = FileList.new(['Cargo.toml', 'fuzz/Cargo.toml', 'spec-runner/Cargo.toml', 'ui-tests/Cargo.toml'])

      failures = pkg_files.map do |file|
        contents = File.read(file)

        if (existing_version = contents.match(regexp))
          File.write(file, contents.gsub(regexp, next_rust_version)) if existing_version != next_rust_version
          next
        end

        puts "Failed to update #{file}, ensure there is a rust-version specified" if Rake.verbose
        file
      end.compact

      raise 'Failed to update some rust-versions' if failures.any?
    end

    desc 'Sync the root rust-toolchain version to CI jobs'
    task :ci do
      workflow_files = FileList.new('.github/workflows/*.yaml')

      workflow_files.each do |file|
        contents = File.read(file)
        contents = contents.gsub(/(toolchain: "?)\d+\.\d+\.\d+("?)/, "\\1#{toolchain_version}\\2")

        File.write(file, contents)
      end
    end
  end
end

KNOWN_WORKSPACE_PREFIXES = %w[
  artichoke
  mezzaluna
  scolapasta
  spinoso
].freeze

KNOWN_FIRST_PARTY = %w[
  boba
  focaccia
  intaglio
  known-folders
  posix-space
  qed
  rand_mt
  raw-parts
  roe
  strftime-ruby
  sysdir
].freeze

namespace :deps do
  desc 'List first-party crate dependencies'
  task :firstparty do
    deps = File.readlines('Cargo.lock', chomp: true)
      .select { |line| line.start_with?('name = ') }
      .map { |line| line.delete_prefix('name = "') }
      .map { |line| line.delete_suffix('"') }
      .select { |dep| KNOWN_FIRST_PARTY.include?(dep) || KNOWN_WORKSPACE_PREFIXES.any? { |prefix| dep.include?(prefix) } }
    puts deps
  end

  desc 'List third-party crate dependencies'
  task :thirdparty do
    deps = File.readlines('Cargo.lock', chomp: true)
      .select { |line| line.start_with?('name = ') }
      .map { |line| line.delete_prefix('name = "') }
      .map { |line| line.delete_suffix('"') }
      .reject { |dep| KNOWN_FIRST_PARTY.include?(dep) }
      .reject { |dep| KNOWN_WORKSPACE_PREFIXES.any? { |prefix| dep.include?(prefix) } }
    puts deps
  end
end
