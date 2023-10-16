use tonic::metadata::{Ascii, KeyAndValueRef, MetadataKey, MetadataMap, MetadataValue};

pub fn print_metadata(metadata: &MetadataMap) {
    metadata_for_each(metadata, |key, value| {
        println!("Ascii: {:?}: {:?}", key, value)
    });
    println!()
}

pub fn merge_metadata(metadata_into: &mut MetadataMap, metadata_from: &MetadataMap) {
    metadata_for_each(&metadata_from, |key, value| {
        if key.to_string().starts_with("x-") {
            metadata_into.insert(key, value.to_owned());
        }
    })
}

fn metadata_for_each<F>(metadata: &MetadataMap, mut f: F)
where
    F: FnMut(&MetadataKey<Ascii>, &MetadataValue<Ascii>),
{
    for key_and_value in metadata.iter() {
        match key_and_value {
            KeyAndValueRef::Ascii(key, value) => f(key, value),
            KeyAndValueRef::Binary(_key, _value) => {
                // TODO ?
            }
        }
    }
}
