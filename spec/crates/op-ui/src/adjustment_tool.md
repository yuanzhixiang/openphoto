# adjustment_tool.rs: Adjustment Brush and retained properties

Adjustment Brush adds a retained adjustment layer with a black mask when the active layer does not match the selected adjustment kind. It reuses a matching adjustment layer. Add paints white and Subtract paints black; Alt reverses the latched operation. Subtract without a matching layer, zero opacity/flow and Quick Mask produce no changes. Strokes honor selection, soft brush diameter, opacity, flow and pressure options through the common paint engine. Locked/hidden targets reject edits. Idle airbrush buildup reuses the existing paint engine.

One changed release records Adjustment Brush Tool. Escape/tool departure restores the original snapshot and mask target; empty new-layer strokes restore without creating history. Optional pink display overlay shows mask coverage through rotation/flip and never changes pixels.

The Properties panel edits retained values of the selected adjustment. The current property controls record every changed update; native slider history grouping and styling remain incomplete. Default effects, brush presets and exact pressure/airbrush timing are application choices, not native acceptance. Select Subject and additional gear options remain unsupported. Source raster pixels are preserved; adjustment settings and masks persist in OpenPhoto's private PSD extension, which Photoshop does not understand as native adjustment layers.

A matching layer with a removed mask receives a new black mask before painting; unchanged strokes restore the original snapshot and mask target regardless of whether a layer was created.
