# 13. Logo and identity brief

**Status:** In progress (I03). Vector assets (`app/public/mark.svg`, `app/public/logo-lockup.svg`, `app/public/icon.svg`) and the 5-panel visual stress-test sheet (`app/public/logo-stress-test.html`) are implemented on the 64-by-64 integer grid with an 8-unit module and 8-unit negative-space channels. Hospital stakeholder visual sign-off and physical wristband sample verification remain open before marking I03 complete.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I03

---

Working name: **MedPulsePH**. Use one recognizable geometric idea rather than a cross, heartbeat, shield, and NFC waves combined. Develop the mark in monochrome and test its physical and digital applications.

| Direction | Single idea | Outcome |
| --- | --- | --- |
| **Clasp — selected** | Asymmetric geometric fastening joint of a wristband (`x=8..28` housing and `x=36..56` latch) separated by an 8-unit negative-space split-vault channel (`x=28..36`). | Selected primary mark; avoids symmetrical chain-link clichés and preserves &ge;8-unit channels at `16×16px` and `8mm` etching scale. |
| Folded M | A continuous folded band forms an M on the 64-by-64 grid. | Documented in `app/public/logo-stress-test.html`; acute interior vertex narrows below 8 units at `8mm` physical engraving scale. |
| Open aperture | A controlled 16-unit opening in an 8-unit solid geometric perimeter. | Documented in `app/public/logo-stress-test.html`; reads more architectural than wearable when used without the wordmark. |

### Produced artifacts and construction rules

- **Grid & Negative Space (`app/public/mark.svg`, `app/public/icon.svg`):** Constructed on a `64×64` integer grid with an `8`-unit stroke module and an `8`-unit (`12.5%` of viewBox) central negative-space channel (`x=28..36`), `12×24` left counter, and `12×8` right latch slot.
- **Horizontal Lockup (`app/public/logo-lockup.svg`):** Single-color wordmark with deliberate kerning and `PH` kept legible without a separate accent badge; minimum clear space is `0.5×` mark height (`32` units) on all sides.
- **5-Panel Visual Stress-Test Sheet (`app/public/logo-stress-test.html`):** Renders Pass 1 (3-direction comparison) and Pass 2 (1-Bit Light `#0A0A0A` on `#FFFFFF`, 1-Bit Dark `#FFFFFF` on `#0A0A0A`, actual `16×16px` and `24×24px` pixel sizes, `64×64` construction-grid overlay, and `8mm` matte NFC wristband stencil mock plus subtraction check).
- **Remaining for I03 completion:** Hospital stakeholder review and physical silicone/TPU wristband engraving sample inspection.
