//! Original byte-level contracts for PDF literal-string end-of-line handling.
#![cfg(feature = "read")]
#![allow(clippy::unwrap_used)]

use craft_pdf::syntax::object::String as PdfString;

fn decoded(token: &[u8]) -> Vec<u8> {
    let mut page = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 10 10] /Probe ".to_vec();
    page.extend_from_slice(token);
    page.extend_from_slice(b" >>");
    let objects = [b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(), b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(), page];
    let mut pdf = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![0];
    for (i, body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        pdf.extend_from_slice(body);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let start = pdf.len();
    pdf.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(format!("trailer << /Size 4 /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n").as_bytes());
    let doc = craft_pdf::open_slice(&pdf, None).unwrap();
    doc.pages()[0].raw().get::<PdfString<'_>>(b"Probe").unwrap().as_bytes().to_vec()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn distinct_end_of_line_markers_preserve_distinct_bytes() {
    let markers: [&[u8]; 3] = [b"\n", b"\r", b"\r\n"];
    for first in markers {
        for second in markers {
            let token = [b"(AA".as_slice(), first, second, b"BBBB)"].concat();
            // A CR followed by LF is one marker, even across these input slices.
            let expected: &[u8] = if first == b"\r" && second == b"\n" { b"AA\nBBBB" } else { b"AA\n\nBBBB" };
            assert_eq!(decoded(&token), expected, "{token:?}");
        }
    }
    assert_eq!(decoded(b"(AA\n\nBBBB)"), decoded(b"<41410a0a42424242>"));
    assert_eq!(decoded(b"(\r\r\n\n)"), b"\n\n\n");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn a_line_continuation_consumes_only_its_own_marker() {
    for marker in [b"\n".as_slice(), b"\r", b"\r\n"] {
        assert_eq!(decoded(&[b"(A\\".as_slice(), marker, b"B)"].concat()), b"AB");
        assert_eq!(decoded(&[b"(A\\".as_slice(), marker, b"\r\nB)"].concat()), b"A\nB");
    }
    assert_eq!(decoded(b"(AAA\\\n\nBBBB)"), decoded(b"<4141410a42424242>"));
    assert_eq!(decoded(b"(\\\r\n\r\n)"), b"\n");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn escapes_binary_bytes_and_parentheses_keep_their_meaning() {
    assert_eq!(decoded(b"(a(b)c\\(d\\)\\\\\\n\\r\\t\\b\\f\\000\\377)"), b"a(b)c(d)\\\n\r\t\x08\x0c\x00\xff");
    assert_eq!(decoded(b"(\x00\xff)"), b"\x00\xff");
    assert_eq!(decoded(b"(plain)"), b"plain");
    assert_eq!(decoded(b"()"), b"");
}
