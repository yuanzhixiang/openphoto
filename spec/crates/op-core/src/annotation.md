# annotation.rs
Notes are persisted document metadata with stable u32 IDs, pixel position, author, UTF-8 text and RGB color. Creation validates in-canvas coordinates. They never enter the image composite and participate in snapshots/undo. Annotation geometry is not remapped by image transforms.
