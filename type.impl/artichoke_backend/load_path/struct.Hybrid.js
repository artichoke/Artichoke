(function() {
    var type_impls = Object.fromEntries([["artichoke_backend",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Hybrid\" class=\"impl\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#11\">Source</a><a href=\"#impl-Debug-for-Hybrid\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"artichoke_backend/load_path/struct.Hybrid.html\" title=\"struct artichoke_backend::load_path::Hybrid\">Hybrid</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#11\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","artichoke_backend::load_path::Adapter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Hybrid\" class=\"impl\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#21-25\">Source</a><a href=\"#impl-Default-for-Hybrid\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"artichoke_backend/load_path/struct.Hybrid.html\" title=\"struct artichoke_backend::load_path::Hybrid\">Hybrid</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#22-24\">Source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; Self</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","artichoke_backend::load_path::Adapter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Hybrid\" class=\"impl\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#27-215\">Source</a><a href=\"#impl-Hybrid\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"artichoke_backend/load_path/struct.Hybrid.html\" title=\"struct artichoke_backend::load_path::Hybrid\">Hybrid</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#35-47\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.new\" class=\"fn\">new</a>() -&gt; Self</h4></section></summary><div class=\"docblock\"><p>Create a new hybrid virtual file system.</p>\n<p>This file system allows access to the host file system with an in-memory\nfile system mounted at <a href=\"artichoke_backend/load_path/constant.RUBY_LOAD_PATH.html\" title=\"constant artichoke_backend::load_path::RUBY_LOAD_PATH\"><code>RUBY_LOAD_PATH</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.resolve_file\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#54-80\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.resolve_file\" class=\"fn\">resolve_file</a>(&amp;self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Check whether <code>path</code> points to a file in the virtual file system and\nreturn the absolute path if it exists.</p>\n<p>This API is infallible and will return <a href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.None\" title=\"variant core::option::Option::None\"><code>None</code></a> for non-existent paths.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_file\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#86-96\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.is_file\" class=\"fn\">is_file</a>(&amp;self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Check whether <code>path</code> points to a file in the virtual file system.</p>\n<p>This API is infallible and will return <code>false</code> for non-existent paths.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.read_file\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#108-119\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.read_file\" class=\"fn\">read_file</a>(&amp;self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/std/io/error/type.Result.html\" title=\"type std::io::error::Result\">Result</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Read file contents for the file at <code>path</code>.</p>\n<p>Returns a byte slice of complete file contents. If <code>path</code> is relative,\nit is absolutized relative to the current working directory of the\nvirtual file system.</p>\n<h5 id=\"errors\"><a class=\"doc-anchor\" href=\"#errors\">§</a>Errors</h5>\n<p>If <code>path</code> does not exist, an <a href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\"><code>io::Error</code></a> with error kind\n<a href=\"https://doc.rust-lang.org/nightly/std/io/error/enum.ErrorKind.html#variant.NotFound\" title=\"variant std::io::error::ErrorKind::NotFound\"><code>io::ErrorKind::NotFound</code></a> is returned.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.write_file\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#134-136\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.write_file\" class=\"fn\">write_file</a>(&amp;mut self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>, buf: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/alloc/borrow/enum.Cow.html\" title=\"enum alloc::borrow::Cow\">Cow</a>&lt;'static, [<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/std/io/error/type.Result.html\" title=\"type std::io::error::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Write file contents into the virtual file system at <code>path</code>.</p>\n<p>Writes the full file contents. If any file contents already exist at\n<code>path</code>, they are replaced. Extension hooks are preserved.</p>\n<p>Only the <a href=\"artichoke_backend/load_path/struct.Memory.html\" title=\"struct artichoke_backend::load_path::Memory\"><code>Memory</code></a> file system at <a href=\"artichoke_backend/load_path/constant.RUBY_LOAD_PATH.html\" title=\"constant artichoke_backend::load_path::RUBY_LOAD_PATH\"><code>RUBY_LOAD_PATH</code></a> is writable.</p>\n<h5 id=\"errors-1\"><a class=\"doc-anchor\" href=\"#errors-1\">§</a>Errors</h5>\n<p>If access to the <a href=\"artichoke_backend/load_path/struct.Memory.html\" title=\"struct artichoke_backend::load_path::Memory\"><code>Memory</code></a> file system returns an error, the error is\nreturned. See <a href=\"artichoke_backend/load_path/struct.Memory.html#method.write_file\" title=\"method artichoke_backend::load_path::Memory::write_file\"><code>Memory::write_file</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_extension\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#142-144\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.get_extension\" class=\"fn\">get_extension</a>(&amp;self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"type\" href=\"artichoke_backend/load_path/type.ExtensionHook.html\" title=\"type artichoke_backend::load_path::ExtensionHook\">ExtensionHook</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Retrieve an extension hook for the file at <code>path</code>.</p>\n<p>This API is infallible and will return <code>None</code> for non-existent paths.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.register_extension\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#159-161\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.register_extension\" class=\"fn\">register_extension</a>(\n    &amp;mut self,\n    path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>,\n    extension: <a class=\"type\" href=\"artichoke_backend/load_path/type.ExtensionHook.html\" title=\"type artichoke_backend::load_path::ExtensionHook\">ExtensionHook</a>,\n) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/std/io/error/type.Result.html\" title=\"type std::io::error::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Write extension hook into the virtual file system at <code>path</code>.</p>\n<p>If any extension hooks already exist at <code>path</code>, they are replaced. File\ncontents are preserved.</p>\n<p>This function writes all extensions to the virtual file system. If the\ngiven path does not map to the virtual file system, the extension is\nunreachable.</p>\n<h5 id=\"errors-2\"><a class=\"doc-anchor\" href=\"#errors-2\">§</a>Errors</h5>\n<p>If the given path does not resolve to the virtual file system, an error\nis returned.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_required\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#167-184\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.is_required\" class=\"fn\">is_required</a>(&amp;self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Check whether a file at <code>path</code> has been required already.</p>\n<p>This API is infallible and will return <code>false</code> for non-existent paths.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.mark_required\" class=\"method\"><a class=\"src rightside\" href=\"src/artichoke_backend/load_path/hybrid.rs.html#196-214\">Source</a><h4 class=\"code-header\">pub fn <a href=\"artichoke_backend/load_path/struct.Hybrid.html#tymethod.mark_required\" class=\"fn\">mark_required</a>(&amp;mut self, path: &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/path/struct.Path.html\" title=\"struct std::path::Path\">Path</a>) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/std/io/error/type.Result.html\" title=\"type std::io::error::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Mark a source at <code>path</code> as required on the interpreter.</p>\n<p>This metadata is used by <code>Kernel#require</code> and friends to enforce that\nRuby sources are only loaded into the interpreter once to limit side\neffects.</p>\n<h5 id=\"errors-3\"><a class=\"doc-anchor\" href=\"#errors-3\">§</a>Errors</h5>\n<p>If <code>path</code> does not exist, an <a href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\"><code>io::Error</code></a> with error kind\n<a href=\"https://doc.rust-lang.org/nightly/std/io/error/enum.ErrorKind.html#variant.NotFound\" title=\"variant std::io::error::ErrorKind::NotFound\"><code>io::ErrorKind::NotFound</code></a> is returned.</p>\n</div></details></div></details>",0,"artichoke_backend::load_path::Adapter"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[15323]}