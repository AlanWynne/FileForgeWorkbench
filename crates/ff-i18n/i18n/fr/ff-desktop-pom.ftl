# French (fr) Primary Option Menu calendar chrome (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/ff-desktop-pom.ftl; only prose translated. The weekday header keeps the
# fixed-width 2-char-column layout (20 chars): French day abbreviations
# Di Lu Ma Me Je Ve Sa. { $value } is a text placeable (pre-formatted in Rust);
# the dotted-leader prose localises and the leader spacing is preserved.

ff-desktop-pom-weekday-header = Di Lu Ma Me Je Ve Sa
ff-desktop-pom-time = Heure . . . . : { $value }
ff-desktop-pom-day-of-year = Jour de l'ann{ "\u00e9" }e : { $value }

# Calendar month names (B084): French, lowercase. Keys identical to
# en/ff-desktop-pom.ftl; accents use the same Fluent { "\uXXXX" } escape as the
# day-of-year key above (localization Req 4.1). 01 = January .. 12 = December.
ff-desktop-pom-month-01 = janvier
ff-desktop-pom-month-02 = f{ "\u00e9" }vrier
ff-desktop-pom-month-03 = mars
ff-desktop-pom-month-04 = avril
ff-desktop-pom-month-05 = mai
ff-desktop-pom-month-06 = juin
ff-desktop-pom-month-07 = juillet
ff-desktop-pom-month-08 = ao{ "\u00fb" }t
ff-desktop-pom-month-09 = septembre
ff-desktop-pom-month-10 = octobre
ff-desktop-pom-month-11 = novembre
ff-desktop-pom-month-12 = d{ "\u00e9" }cembre
