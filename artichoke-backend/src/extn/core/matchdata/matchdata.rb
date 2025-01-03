# frozen_string_literal: true

class MatchData
  def ==(other)
    return false unless other.is_a?(MatchData)
    return false unless string == other.string
    return false unless regexp == other.regexp
    return false unless offset(0) == other.offset(0)

    true
  end

  def eql?(other)
    self == other
  end

  def inspect
    # All interpolations below must call `#inspect` on the captures to ensure
    # that the captures are formatted correctly if they contain binary content,
    # control characters, such as a null byte, or legacy escape sequences.
    #
    # ```console
    # [3.3.6] > m = "\0".match /(.)/
    # => #<MatchData "\u0000" 1:"\u0000">
    # => #<MatchData "\t" 1:"\t">
    # [3.3.6] > m = "\e".match /(.)/
    # => #<MatchData "\e" 1:"\e">
    # [3.3.6] > m = "\xFF".b.match /(.)/
    # => #<MatchData "\xFF" 1:"\xFF">
    # ```
    s = %(#<MatchData #{self[0].inspect})

    if names.empty?
      captures.each_with_index do |capture, index|
        s << %( #{index + 1}:#{capture.inspect})
      end
    else
      names.each do |name|
        capture = self[name]
        s << %( #{name}:#{capture.inspect})
      end
    end
    s << '>'
  end

  def values_at(*indexes)
    indexes.map { |index| self[index] }.flatten
  end
end
