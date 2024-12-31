# frozen_string_literal: true

def spec
  setup

  inspect_no_garbage

  true
end

def setup
  Kernel.alias_method :original_puts, :puts
  Kernel.define_method(:puts) do |*args|
    raise "puts called with #{args.inspect}"
  end
end

def inspect_no_garbage
  m = 'a'.match(/./)
  raise 'MatchData#inspect does not contain garbage' unless m.inspect == '#<MatchData "a">'

  m = 'a'.match(/(.)/)
  raise 'MatchData#inspect does not contain garbage with capturing groups' unless m.inspect == '#<MatchData "a" 1:"a">'
end

spec if $PROGRAM_NAME == __FILE__
