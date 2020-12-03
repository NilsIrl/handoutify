use lopdf::{Document, Object};

pub fn handoutify(doc: &mut Document) {
    let catalog = doc.catalog().unwrap();
    let nums = match catalog.get(b"PageLabels").unwrap() {
        Object::Dictionary(dictionary) => dictionary,
        _ => unreachable!(),
    }
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

    doc.delete_pages(&pages);
}
