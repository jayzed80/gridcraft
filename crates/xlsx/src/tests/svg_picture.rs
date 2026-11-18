//! An SVG picture is stored as a raster `a:blip` with the SVG source in its `asvg:svgBlip`
//! extension. A save keeps both parts, not just the raster fallback (#4481).

use std::io::Read;

use super::minimal;
use crate::{read_xlsx, write_xlsx};

const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"#;
const PNG: &str = "PNG fallback bytes";

fn svg_picture_book() -> Vec<u8> {
    let drawing = r#"<xdr:wsDr xmlns:xdr="http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
      <xdr:twoCellAnchor editAs="oneCell"><xdr:from><xdr:col>1</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>1</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:from><xdr:to><xdr:col>3</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>6</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:to>
        <xdr:pic><xdr:nvPicPr><xdr:cNvPr id="2" name="Graphic 1" descr="vector"/><xdr:cNvPicPr/></xdr:nvPicPr><xdr:blipFill><a:blip r:embed="rId1"><a:extLst><a:ext uri="{96DAC541-7B7A-43D3-8B79-37D633B846F1}"><asvg:svgBlip xmlns:asvg="http://schemas.microsoft.com/office/drawing/2016/SVG/main" r:embed="rId2"/></a:ext></a:extLst></a:blip><a:stretch><a:fillRect/></a:stretch></xdr:blipFill><xdr:spPr/></xdr:pic><xdr:clientData/></xdr:twoCellAnchor>
    </xdr:wsDr>"#;
    let drels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image2.svg"/></Relationships>"#;
    let srels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing" Target="../drawings/drawing1.xml"/></Relationships>"#;
    minimal(
        r#"<sheetData/><drawing r:id="rId1"/>"#,
        &[
            ("xl/worksheets/_rels/sheet1.xml.rels", srels),
            ("xl/drawings/drawing1.xml", drawing),
            ("xl/drawings/_rels/drawing1.xml.rels", drels),
            ("xl/media/image1.png", PNG),
            ("xl/media/image2.svg", SVG),
        ],
        "",
        "",
    )
}

fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i).unwrap();
            let mut d = vec![];
            f.read_to_end(&mut d).unwrap();
            (f.name().to_string(), d)
        })
        .collect()
}

fn text(parts: &[(String, Vec<u8>)], name: &str) -> String {
    let (_, d) = parts.iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("no {name}"));
    String::from_utf8(d.clone()).unwrap()
}

/// The relationship target that `rid` names in `rels`, as a package part name under `xl/`.
fn target(rels: &str, rid: &str) -> String {
    let rel = rels.split("<Relationship ").find(|r| r.contains(&format!("Id=\"{rid}\""))).unwrap_or_else(|| panic!("no {rid} in {rels}"));
    let t = rel.split("Target=\"").nth(1).unwrap().split('"').next().unwrap();
    format!("xl/{}", t.trim_start_matches("../"))
}

fn attr_after<'a>(xml: &'a str, marker: &str) -> &'a str {
    let rest = xml.split(marker).nth(1).unwrap_or_else(|| panic!("no {marker} in {xml}"));
    rest.split("r:embed=\"").nth(1).unwrap().split('"').next().unwrap()
}

#[test]
fn an_svg_picture_keeps_its_svg_source_and_png_fallback_on_save() {
    let (wb, _) = read_xlsx(&svg_picture_book()).unwrap();
    let img = &wb.sheet(0).unwrap().images[0];
    assert_eq!(img.data.as_slice(), PNG.as_bytes());
    assert_eq!(img.svg.as_deref().map(Vec::as_slice), Some(SVG.as_bytes()));

    let out = parts(&write_xlsx(&wb).unwrap());
    let drawing = text(&out, "xl/drawings/drawing1.xml");
    let rels = text(&out, "xl/drawings/_rels/drawing1.xml.rels");
    // The raster fallback and the SVG source each have their own media part and relationship.
    let png_part = target(&rels, attr_after(&drawing, "<a:blip "));
    let svg_part = target(&rels, attr_after(&drawing, "<asvg:svgBlip "));
    assert!(drawing.contains("<a:ext uri=\"{96DAC541-7B7A-43D3-8B79-37D633B846F1}\">"), "{drawing}");
    assert!(svg_part.ends_with(".svg"), "{svg_part}");
    assert_eq!(text(&out, &png_part), PNG);
    assert_eq!(text(&out, &svg_part), SVG);
    assert!(text(&out, "[Content_Types].xml").contains("<Default Extension=\"svg\" ContentType=\"image/svg+xml\"/>"));

    // It reads back the same, and a PNG-only picture is still written without the extension.
    let back = read_xlsx(&write_xlsx(&wb).unwrap()).unwrap().0;
    assert_eq!(back.sheet(0).unwrap().images[0].svg, img.svg);
    let mut plain = wb.clone();
    std::sync::Arc::make_mut(&mut plain.sheets[0]).images[0].svg = None;
    let out = parts(&write_xlsx(&plain).unwrap());
    assert!(!text(&out, "xl/drawings/drawing1.xml").contains("svgBlip"));
    assert!(!out.iter().any(|(n, _)| n.ends_with(".svg")));
}
