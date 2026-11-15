## #976 - Retain the Heidenhain inch refusal pending cycle feed evidence

**Decided:** Keep `programUnits = "mm"` and the current named refusal for an
inch document. Escalate the issue for the remaining safety specification.
The held pin `heidenhain_inch_linear_uses_g70_and_tenths_inch_feed` fails on
unchanged upstream code because the post refuses inch documents. No
production code or golden changed.

**Why:** The [TNC 640 ISO programming manual, 892909-25,
section 5.1, page 200](https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-07/diniso/892909-25.pdf)
states the tenth-inch-per-minute rule for path feeds in positioning and
tool blocks. The [cycle programming manual, 892905-25, pages 82 and
97–98](https://content.heidenhain.de/doku/tnc_guide/pdf_files/TNC640/34059x-07/zyklen/892905-25.pdf)
lists numeric cycle feeds such as Q206 and Q253 in mm/min, without an inch
program rule. Extending the path-feed rule to cycle parameters would guess
at a factor of ten in cutting feed. The issue requires those cycle feeds
to use the unit the published manual states.

**Considered and dropped:** Guessing tenth-inch-per-minute cycle feeds from
the path-feed rule does not establish the cycle contract. Adding inch
support only for motion and refusing inch cycles does not meet the issue's
done criteria or its required inch drilling golden. Neither is a complete
safe fix for this issue.

**Cost to change later:** Find published TNC 640 evidence for numeric inch
cycle feeds and representable pitch/feed resolution. Then implement unit
conversion, extend pins to cycles and precision refusals, generate inch
goldens, and prove existing mm goldens remain byte-identical. The held
linear pin can be reused after that evidence is available.
