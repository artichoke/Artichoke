# frozen_string_literal: true

def spec
  setup

  inspect_no_garbage
  inspect_with_numbered_captures
  inspect_with_named_captures
  inspect_capture_control_character

  true
end

def setup
  Kernel.alias_method :original_puts, :puts
  Kernel.define_method(:puts) do |*args|
    raise "puts called with #{args.inspect}"
  end
end

def expected_failure_due_to_implementation_gaps(expected_message)
  yield
rescue StandardError => e
  raise e unless e.message == expected_message
else
  raise "Expected failure due to implementation gaps but did not get #{expected_message}"
end

def inspect_no_garbage
  m = 'a'.match(/./)
  raise 'MatchData#inspect does not contain garbage' unless m.inspect == '#<MatchData "a">'

  m = 'a'.match(/(.)/)
  raise 'MatchData#inspect does not contain garbage with capturing groups' unless m.inspect == '#<MatchData "a" 1:"a">'
end

def inspect_with_numbered_captures
  m = 'a'.match(/(.)/)
  raise 'MatchData#inspect with numbered captures' unless m.inspect == '#<MatchData "a" 1:"a">'

  m = 'a'.match(/(.)(.)?/)
  raise 'MatchData#inspect with numbered captures' unless m.inspect == '#<MatchData "a" 1:"a" 2:nil>'
end

def inspect_with_named_captures
  m = 'a'.match(/(?<dot>.)/)
  raise 'MatchData#inspect with named captures' unless m.inspect == '#<MatchData "a" dot:"a">'

  m = 'a'.match(/(?<dot>.)(?<nope>.)?/)
  raise 'MatchData#inspect with missing named captures' unless m.inspect == '#<MatchData "a" dot:"a" nope:nil>'
end

def inspect_capture_control_character
  # FIXME: Artichoke does not properly format control characters in UTF-8 strings.
  # Artichoke currently yields this string: #<MatchData "\x00" 1:"\x00">
  expected_failure_due_to_implementation_gaps('MatchData#inspect with NUL character') do
    m = "\0".match(/(.)/)
    raise 'MatchData#inspect with NUL character' unless m.inspect == '#<MatchData "\u0000" 1:"\u0000">'
  end

  m = "\t".match(/(.)/)
  raise 'MatchData#inspect with control character' unless m.inspect == '#<MatchData "\t" 1:"\t">'

  m = "\e".match(/(.)/)
  raise 'MatchData#inspect with control character' unless m.inspect == '#<MatchData "\e" 1:"\e">'

  # FIXME: Artichoke does not yet use binary regexps on binary strings.
  expected_failure_due_to_implementation_gaps('regex crate utf8 backend for Regexp only supports UTF-8 haystacks') do
    m = "\xFF".b.match(/(.)/)
    raise 'MatchData#inspect with invalid UTF-8' unless m.inspect == '#<MatchData "\xFF" 1:"\xFF">'
  end
end

spec if $PROGRAM_NAME == __FILE__
