---
id: tweede-engine-geeft-dezelfde-uitkomst
titel: Een tweede engine geeft dezelfde uitkomst
aanname: |-
  Een tweede implementatie kan op basis van het schema en de conformance-suite
  dezelfde uitkomsten geven als de Rust-engine, zonder de code van die engine te
  lezen.
impact: 4
zekerheid: 3
onderbouwing: |-
  De conformance-suite staat er deels. De rekenregels voor afronding, deling en
  datumrekenen zijn nog niet allemaal vastgelegd.
toetsing: |-
  De conformance-suite laten draaien tegen een implementatie die door een
  ander team is gebouwd.
werkpakketIds:
  - specificaties-v-beproeving-van-de-taal
  - interoperabiliteit-van-regeltalen-en-analysemethoden
---
