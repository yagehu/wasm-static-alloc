struct Entry {
    alignment: usize,
    data:      Vec<u8>,
}

pub struct Stack {
    entries: Vec<Entry>,
}

impl Stack {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn push<D>(&mut self, data: D, alignment: usize)
    where
        D: IntoIterator<Item = u8>,
        D::IntoIter: ExactSizeIterator,
    {
        self.entries.push(Entry {
            alignment,
            data: data.into_iter().collect(),
        });
    }

    pub fn iter_entry(&self) -> DataEntryIterator {
        DataEntryIterator {
            stack:           self,
            next_idx:        0,
            prev_end_offset: 0,
        }
    }

    pub fn memory_pages_needed(&self) -> usize {
        const PAGE_SIZE_BYTES: usize = 65536;

        let entry = match self.iter_entry().last() {
            | None => return 0,
            | Some(e) => e,
        };
        let data_len = entry.offset + entry.data.len();
        let extra_page_needed = data_len % PAGE_SIZE_BYTES > 0;

        (entry.offset + entry.data.len()) / PAGE_SIZE_BYTES + if extra_page_needed { 1 } else { 0 }
    }
}

pub struct DataEntryIterator<'a> {
    stack:           &'a Stack,
    next_idx:        usize,
    prev_end_offset: usize,
}

impl<'a> Iterator for DataEntryIterator<'a> {
    type Item = DataEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.stack.entries.get(self.next_idx)?;
        let offset = self.prev_end_offset
            + (entry.alignment - self.prev_end_offset % entry.alignment) % entry.alignment;

        self.next_idx += 1;
        self.prev_end_offset = offset + entry.data.len();

        Some(DataEntry {
            offset,
            data: &entry.data,
        })
    }
}

impl ExactSizeIterator for DataEntryIterator<'_> {
    fn len(&self) -> usize {
        self.stack.entries.len()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DataEntry<'a> {
    pub offset: usize,
    pub data:   &'a [u8],
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok() {
        let mut stack = Stack::default();

        stack.push([1; 4], 4);
        stack.push([2; 5], 1);
        stack.push([3; 4], 4);
        stack.push([4; 65536], 8);

        let mut entries = stack.iter_entry();
        let expected_entries = [
            DataEntry {
                offset: 0,
                data:   &[1; 4],
            },
            DataEntry {
                offset: 4,
                data:   &[2; 5],
            },
            DataEntry {
                offset: 12,
                data:   &[3; 4],
            },
            DataEntry {
                offset: 16,
                data:   &[4; 65536],
            },
        ];

        for expected_entry in expected_entries {
            let entry = entries.next().unwrap();

            assert_eq!(entry, expected_entry);
        }

        assert_eq!(stack.memory_pages_needed(), 2);
    }
}
