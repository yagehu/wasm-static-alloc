pub type EntryId = usize;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct DataEntry {
    pub alignment: usize,
    pub offset:    usize,
    pub data:      Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Stack {
    entries: Vec<DataEntry>,
}

impl Stack {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn get_entry(&self, id: EntryId) -> Option<&DataEntry> {
        self.entries.get(id)
    }

    pub fn get_entry_mut(&mut self, id: EntryId) -> Option<&mut DataEntry> {
        self.entries.get_mut(id)
    }

    pub fn push<D>(&mut self, data: D, alignment: usize) -> (EntryId, usize)
    where
        D: IntoIterator<Item = u8>,
        D::IntoIter: ExactSizeIterator,
    {
        let prev_end_offset = self
            .entries
            .last()
            .map(|entry| entry.offset + entry.data.len())
            .unwrap_or_default();
        let offset = prev_end_offset + (alignment - prev_end_offset % alignment) % alignment;

        self.entries.push(DataEntry {
            alignment,
            offset,
            data: data.into_iter().collect(),
        });

        (self.entries.len() - 1, offset)
    }

    pub fn iter_entries(&self) -> impl ExactSizeIterator<Item = &DataEntry> {
        self.entries.iter()
    }

    pub fn memory_pages_needed(&self) -> usize {
        const PAGE_SIZE_BYTES: usize = 65536;

        let entry = match self.iter_entries().last() {
            | None => return 0,
            | Some(e) => e,
        };
        let data_len = entry.offset + entry.data.len();
        let extra_page_needed = data_len % PAGE_SIZE_BYTES > 0;

        (entry.offset + entry.data.len()) / PAGE_SIZE_BYTES + if extra_page_needed { 1 } else { 0 }
    }
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

        let mut entries = stack.iter_entries();
        let expected_entries = [
            DataEntry {
                alignment: 4,
                offset:    0,
                data:      vec![1; 4],
            },
            DataEntry {
                alignment: 1,
                offset:    4,
                data:      vec![2; 5],
            },
            DataEntry {
                alignment: 4,
                offset:    12,
                data:      vec![3; 4],
            },
            DataEntry {
                alignment: 8,
                offset:    16,
                data:      vec![4; 65536],
            },
        ];

        for expected_entry in expected_entries {
            let entry = entries.next().unwrap();

            assert_eq!(entry, &expected_entry);
        }

        assert_eq!(stack.memory_pages_needed(), 2);
    }
}
