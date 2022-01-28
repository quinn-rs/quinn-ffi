(function() {var implementors = {};
implementors["quinn_proto"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"quinn_proto/struct.VarInt.html\" title=\"struct quinn_proto::VarInt\">VarInt</a>","synthetic":false,"types":["quinn_proto::varint::VarInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u128.html\">u128</a>&gt; for <a class=\"struct\" href=\"quinn_proto/struct.VarInt.html\" title=\"struct quinn_proto::VarInt\">VarInt</a>","synthetic":false,"types":["quinn_proto::varint::VarInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt; for <a class=\"struct\" href=\"quinn_proto/struct.VarInt.html\" title=\"struct quinn_proto::VarInt\">VarInt</a>","synthetic":false,"types":["quinn_proto::varint::VarInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/time/struct.Duration.html\" title=\"struct core::time::Duration\">Duration</a>&gt; for <a class=\"struct\" href=\"quinn_proto/struct.IdleTimeout.html\" title=\"struct quinn_proto::IdleTimeout\">IdleTimeout</a>","synthetic":false,"types":["quinn_proto::config::IdleTimeout"]}];
implementors["rustls"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"rustls/internal/msgs/message/struct.PlainMessage.html\" title=\"struct rustls::internal::msgs::message::PlainMessage\">PlainMessage</a>&gt; for <a class=\"struct\" href=\"rustls/internal/msgs/message/struct.Message.html\" title=\"struct rustls::internal::msgs::message::Message\">Message</a>","synthetic":false,"types":["rustls::msgs::message::Message"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;&amp;'_ <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt; for <a class=\"enum\" href=\"rustls/client/enum.ServerName.html\" title=\"enum rustls::client::ServerName\">ServerName</a>","synthetic":false,"types":["rustls::client::client_conn::ServerName"]}];
implementors["tinyvec"] = [{"text":"impl&lt;T, A&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.slice.html\">&amp;'_ [T]</a>&gt; for <a class=\"struct\" href=\"tinyvec/struct.ArrayVec.html\" title=\"struct tinyvec::ArrayVec\">ArrayVec</a>&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"tinyvec/trait.Array.html\" title=\"trait tinyvec::Array\">Array</a>&lt;Item = T&gt;,&nbsp;</span>","synthetic":false,"types":["tinyvec::arrayvec::ArrayVec"]}];
implementors["webpki"] = [{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">&amp;'a [</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">]</a>&gt; for <a class=\"struct\" href=\"webpki/struct.EndEntityCert.html\" title=\"struct webpki::EndEntityCert\">EndEntityCert</a>&lt;'a&gt;","synthetic":false,"types":["webpki::end_entity::EndEntityCert"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/time/struct.SystemTime.html\" title=\"struct std::time::SystemTime\">SystemTime</a>&gt; for <a class=\"struct\" href=\"webpki/struct.Time.html\" title=\"struct webpki::Time\">Time</a>","synthetic":false,"types":["webpki::time::Time"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()