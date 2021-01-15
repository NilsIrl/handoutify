use lopdf::{Document, Object};

const PAGE_LABELS: &[u8; 10] = b"PageLabels";

pub fn handoutify(doc: &mut Document) {
    // Get catalog manually https://docs.rs/lopdf/0.26.0/src/lopdf/document.rs.html#148-153
    let catalog_id = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    let nums = doc
        .get_object(catalog_id)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(PAGE_LABELS)
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Nums")
        .unwrap();

    let arr = match nums {
        Object::Array(array) => array,
        _ => unreachable!(),
    };

    let mut prev = 0;

    let page_nb = doc.page_iter().size_hint().0 as u32;
    let mut pages = Vec::new();

    for x in arr
        .iter()
        .step_by(2)
        .skip(1)
        .map(|object| match object {
            Object::Integer(index) => *index as u32,
            _ => unreachable!(),
        })
        .chain(std::iter::once(page_nb))
    {
        pages.extend((prev + 1)..x);
        prev = x;
    }

    doc.get_object_mut(catalog_id)
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .remove(PAGE_LABELS)
        .unwrap();
    doc.delete_pages(&pages);
}
