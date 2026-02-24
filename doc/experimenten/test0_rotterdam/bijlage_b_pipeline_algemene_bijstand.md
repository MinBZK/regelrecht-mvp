

## Bijlage B — Processpecificatie verlening algemene bijstand Rotterdam (geldend per 19-02-2026)

Deze bijlage specificeert het volledige beslisproces voor de verlening van algemene bijstand (art. 19-23 Participatiewet) in Rotterdam, stap voor stap, met per beslispunt de exacte wettelijke grondslag, de geldende parameters per 01-01-2026, en de classificatie als machine-uitvoerbaar (M), gedeeltelijk machine-uitvoerbaar (G) of menselijk (H). De specificatie betreft uitsluitend de verlening van reguliere algemene bijstand aan personen van 18 jaar tot de AOW-leeftijd, exclusief IOAW, IOAZ, Bbz en bijzondere bijstand.

Juridisch kader: Participatiewet geldend per 01-01-2026 (BWBR0015703, inclusief PiB fase 1, Stb. 2025, 312/313). Rotterdamse regelgeving: Participatieverordening Rotterdam 2015 (CVDR361766/5), Nadere regels voorzieningen (CVDR703171/2), Verordening maatregelen en handhaving (CVDR348678/4), Beleidsregels bijzondere bijstand 2024 (CVDR719087).

---

### B.1 STAP 1 — Melding (registratie van het verzoek)

**Wettelijke grondslag:** art. 41 lid 1 Pw (melding bij UWV of college), art. 44 lid 1-2 Pw (ingangsdatum = meldingsdatum).

**Trigger:** een persoon meldt zich bij het UWV-werkbedrijf (werk.nl) of rechtstreeks bij de gemeente Rotterdam als werkzoekende en/of bijstandsaanvrager.

**Processtap 1.1 — Vastlegging meldingsdatum [M]**

Het systeem registreert de meldingsdatum als dag waarop de persoon zich meldt. Dit is de potentiële ingangsdatum van de bijstand (art. 44 lid 1 Pw). Vanaf PiB fase 1 kan bijstand met terugwerkende kracht tot maximaal drie maanden vóór de meldingsdatum worden verleend, mits de aanvrager aannemelijk maakt dat hij in die periode al bijstandsbehoeftig was (art. 44 lid 4 Pw nieuw). De beoordeling van de terugwerkende kracht is een menselijke beslissing (H).

**Processtap 1.2 — Leeftijdscheck: zoektermijn jongeren < 27 [M/H]**

Beslisregel:
- INPUT: geboortedatum (uit BRP of opgave)
- BEREKENING: leeftijd op meldingsdatum
- IF leeftijd < 27 THEN zoektermijn van vier weken is van toepassing (art. 41 lid 4 Pw), TENZIJ het college oordeelt dat de omstandigheden van de belanghebbende of het gezin aanleiding geven om de aanvraag eerder in behandeling te nemen (art. 41 lid 11 Pw, PiB fase 1 — kan-bepaling) [H]
- IF leeftijd ≥ 27 THEN geen wettelijke zoektermijn; in de Rotterdamse praktijk wordt een inspanningsperiode gehanteerd, maar deze is geen wettelijke vereiste [H — Rotterdam-specifiek]

De zoektermijn-uitzondering is een kan-bepaling. De Handreiking Participatiewet in Balans (Stimulansz, november 2025) noemt als richtgevende groepen voor wie de uitzondering passend kan zijn: jongeren uit het praktijkonderwijs of VSO (tot een jaar na uitschrijving), jongeren met een medische urenbeperking (art. 6b Pw), en jongeren die tot de doelgroep loonkostensubsidie behoren. Rotterdam heeft per 19-02-2026 geen lokaal beleid gepubliceerd dat preciseert wanneer de uitzondering wordt toegepast (gap 18).

**Processtap 1.3 — Verwijzing bij regulier onderwijs [M/H]**

IF leeftijd < 27 AND persoon volgt regulier onderwijs (te verifiëren via DUO) THEN geen recht op algemene bijstand (art. 13 lid 2 sub c Pw), verwijzing naar studiefinanciering. Uitzondering: de opleiding is geen voltijds regulier onderwijs, of de student valt onder een hardheidsclausule.

**Output stap 1:** meldingsdatum geregistreerd, leeftijdscategorie bepaald (<27 of ≥27), eventuele zoektermijn geactiveerd, doorverwijzing bij regulier onderwijs.

---

### B.2 STAP 2 — Aanvraag en dossiervorming

**Wettelijke grondslag:** art. 41 Pw (aanvraag), art. 43 lid 1 Pw (vaststelling recht door college), art. 43a Pw [PiB-F1] (identificatie met rijbewijs/DigiD), art. 17 Pw (inlichtingenplicht), art. 4:2 Awb (verplichting gegevens verstrekken), art. 4:13 Awb (beslistermijn 8 weken).

**Trigger:** na afloop van de eventuele zoektermijn (stap 1.2), of direct bij personen ≥ 27 jaar, dient de persoon een formele aanvraag in.

**Processtap 2.1 — Identificatie [M/H]**

Per 01-01-2026 (PiB fase 1) zijn de volgende identificatiemiddelen geldig: geldig Nederlands paspoort, geldige Nederlandse identiteitskaart, geldig EU-rijbewijs (nieuw per PiB), of identificatie via DigiD (nieuw per PiB). Het systeem controleert de geldigheid van het identiteitsdocument. De vaststelling van de identiteit zelf is machine-uitvoerbaar als het document digitaal wordt geverifieerd; bij twijfel over de identiteit is menselijke beoordeling vereist.

**Processtap 2.2 — Verkorte aanvraag-check [M]**

Beslisregel:
- INPUT: BSN, datum einde vorige bijstandsverlening (uit Socrates)
- IF persoon had eerder bijstand bij gemeente Rotterdam AND einde vorige bijstandsverlening < 12 maanden geleden (PiB fase 1; was 6 maanden) THEN verkorte aanvraagprocedure: gemeente mag uitgaan van reeds bekende gegevens uit het dossier
- ELSE standaard aanvraagprocedure

**Processtap 2.3 — Gegevens verzamelen [M/H]**

Het systeem raadpleegt via Suwinet: BRP-gegevens (persoonsgegevens, adres, huishoudsamenstelling), UWV-gegevens (dienstverbanden, lonen, WW-uitkering), SVB-gegevens (AOW, kinderbijslag, Anw), Belastingdienstgegevens (toeslagen, belastbaar inkomen). Aanvullend levert de aanvrager zelf aan: bankafschriften (alle rekeningen, laatste drie maanden), gegevens over vermogensbestanddelen (spaargeld, beleggingen, auto, kostbare bezittingen), huurcontract of hypotheekakte, informatie over eventueel parttime inkomen, bewijsstukken schulden (bij beroep op schuldenaftrek in vermogenstoets).

**Processtap 2.4 — Voorschot [M]**

Beslisregel:
- IF aanvraag is ingediend AND aanvraag is niet kennelijk ongegrond THEN voorschot binnen vier weken na aanvraag (art. 52 lid 1 Pw)
- Hoogte voorschot: minimaal 95% van de verwachte bijstandsuitkering (PiB fase 1; was 90%)
- BEREKENING: verwachte_norm × 0,95 = voorschotbedrag
- Het voorschot wordt verrekend met de eerste reguliere uitkeringsbetaling of teruggevorderd bij afwijzing

Per 01-01-2026 parameters: voorschot alleenstaande 21+ = 0,95 × €1.401,50 = €1.331,43 (afgerond); voorschot gehuwden 21+ = 0,95 × €2.002,13 = €1.902,02 (afgerond). De exacte berekening hangt af van de verwachte leefvorm en eventuele kostendelersnorm.

**Processtap 2.5 — Termijnbewaking [M]**

Het systeem bewaakt de wettelijke beslistermijn van acht weken (art. 4:13 Awb). Bij overschrijding is de gemeente van rechtswege in gebreke (art. 4:17 Awb, dwangsom bij niet tijdig beslissen).

**Output stap 2:** formele aanvraag geregistreerd, identiteit geverifieerd, gegevens verzameld, eventueel voorschot berekend en uitbetaald, beslistermijn gestart.

---

### B.3 STAP 3 — Rechtsvaststelling: is er recht op bijstand?

**Wettelijke grondslag:** art. 11 Pw (rechthebbenden), art. 13 Pw (uitsluitingsgronden), art. 15 Pw (voorliggende voorziening), art. 16 Pw (buitenlanders).

Dit is de juridische poortwachter: vóórdat de hoogte van de uitkering wordt berekend, moet worden vastgesteld dat de aanvrager überhaupt recht heeft op bijstand. Deze stap bestaat uit drie sequentiële checks.

**Processtap 3.1 — Rechthebbende-check (art. 11 Pw) [M/H]**

Beslisregel:
- IF persoon is Nederlander (art. 11 lid 1 sub a) OR persoon is daarmee gelijkgesteld (art. 11 lid 2-3: verblijfsvergunning, EU-onderdaan met rechtmatig verblijf, etc.) THEN door naar 3.2
- IF persoon verblijft niet rechtmatig in Nederland THEN afwijzen (art. 11 lid 1, geen recht)
- IF persoon is jonger dan 18 jaar THEN afwijzen (geen zelfstandig recht op algemene bijstand)
- IF persoon heeft AOW-leeftijd bereikt THEN verwijzen naar AIO-aanvulling (SVB)

De nationaliteitscheck is machine-uitvoerbaar via BRP. De verblijfsrechtelijke status vereist soms menselijke beoordeling (complexe verblijfssituaties, lopende procedures).

**Processtap 3.2 — Uitsluitingsgronden-check (art. 13 Pw) [M/H]**

Beslisregel — geen recht op bijstand indien:
- persoon verblijft in een inrichting (art. 13 lid 1 sub a) — check via BRP/opgave [G]
- persoon is gedetineerd (art. 13 lid 1 sub a) — check via Suwinet/VRIS [M]
- persoon heeft beroep op een voorliggende voorziening (art. 15) — zie stap 3.3 [H]
- persoon is jonger dan 27 en volgt regulier onderwijs met studiefinanciering (art. 13 lid 2 sub c) — check via DUO [M]
- persoon verblijft in het buitenland (art. 13 lid 1 sub d) — check via reisgegevens/opgave [H]
- persoon doet onvoldoende moeite om te voldoen aan de Nederlandse taaleis (art. 18b Pw) — beoordeling [H]

**Processtap 3.3 — Voorliggende-voorziening-check (art. 15 Pw) [H]**

De gemeente toetst of er een voorliggende voorziening is die de kosten dekt waarvoor bijstand wordt gevraagd. Voorliggende voorzieningen zijn onder meer: WW-uitkering (UWV), WIA/WAO-uitkering (UWV), Wajong (UWV), Anw-uitkering (SVB), studiefinanciering (DUO), kinderopvangtoeslag (Belastingdienst), zorgtoeslag en huurtoeslag (Belastingdienst). De check of er een adequate voorliggende voorziening beschikbaar is, vereist in veel gevallen menselijke beoordeling (met name bij gedeeltelijke arbeidsongeschiktheid, afwijzing van andere uitkeringen, en combinaties van voorzieningen). De raadpleging van Suwinet geeft inzicht in lopende uitkeringen en is machine-uitvoerbaar.

**Output stap 3:** beslissing "recht op bijstand JA/NEE" met motivering per uitsluitingsgrond.

---

### B.4 STAP 4 — Leefvormbepaling

**Wettelijke grondslag:** art. 3 Pw (gezamenlijke huishouding), art. 4 Pw (definities alleenstaande, alleenstaande ouder, gezin), art. 19a Pw (kostendeler), art. 22a Pw (kostendelersnorm).

De leefvorm is de eerste en meest bepalende variabele in de normberekening. Alles — norm, vermogensgrens, toepasselijke toeslagen — hangt hiervan af.

**Processtap 4.1 — Basisleefvorm bepalen (art. 3-4 Pw) [M/H]**

Beslisboom op basis van BRP-gegevens en aanvullende verklaringen:

Eerste niveau — burgerlijke staat en samenwoning:
- IF gehuwd (art. 3 lid 2 sub a Pw: huwelijk, geregistreerd partnerschap) AND niet duurzaam gescheiden AND beide partners in zelfde woning THEN leefvorm = GEHUWDEN
- IF ongehuwd AND geen gezamenlijke huishouding (art. 3 lid 3 Pw) AND geen ten laste komende kinderen < 18 THEN leefvorm = ALLEENSTAANDE
- IF ongehuwd AND geen gezamenlijke huishouding AND wél ten laste komende kinderen < 18 THEN leefvorm = ALLEENSTAANDE_OUDER
- IF ongehuwd AND gezamenlijke huishouding (art. 3 lid 3 Pw: twee personen die hun hoofdverblijf in dezelfde woning hebben én blijk geven zorg te dragen voor elkaar door middel van bijdragen in kosten van huishouding of anderszins) THEN leefvorm = GEHUWDEN (gelijkgesteld)

De vaststelling van een gezamenlijke huishouding vereist in veel gevallen menselijke beoordeling. Het systeem kan op basis van BRP-gegevens (meerdere personen ingeschreven op hetzelfde adres) een vermoeden genereren, maar de feitelijke beoordeling of sprake is van "zorg dragen voor elkaar" is niet machine-uitvoerbaar.

**Processtap 4.2 — Zorgbehoefte-uitzondering (art. 3 lid 2 sub a Pw, PiB fase 1) [H]**

Nieuw per 01-01-2026: IF twee personen samenwonen EN de samenwoning is het gevolg van zorgbehoefte van of zorgverlening door één van beiden THEN geen gezamenlijke huishouding, ongeacht bloedverwantschap. De beoordeling of "sprake is van een zorgbehoefte die de aanleiding vormt om samen te wonen" is volledig menselijk. Rotterdam heeft hiervoor geen beoordelingskader gepubliceerd (gap 23).

**Processtap 4.3 — Kinderen [M]**

Beslisregel:
- IF leefvorm = GEHUWDEN AND er zijn ten laste komende kinderen < 18 jaar (BRP) THEN subcategorie = GEHUWDEN_MET_KINDEREN
- IF leefvorm = GEHUWDEN AND geen ten laste komende kinderen < 18 jaar THEN subcategorie = GEHUWDEN_ZONDER_KINDEREN

Dit onderscheid is relevant voor de individuele inkomenstoeslag (§2.1 hoofddocument: €300 vs. €400) maar niet voor de bijstandsnorm zelf (art. 21 Pw: gehuwdennorm is gelijk ongeacht kinderen; kinderbijslag via SVB is de voorliggende voorziening).

**Processtap 4.4 — Leeftijdscategorie [M]**

Beslisregel op basis van geboortedatum:
- IF leeftijd 18-20 THEN jongerennorm van toepassing (art. 20 Pw, gewijzigd PiB fase 1: aanvullende bijstand voor uitwonende 18-20-jarigen is voortaan algemene bijstand)
- IF leeftijd 21 tot AOW-leeftijd THEN reguliere norm
- IF leeftijd ≥ AOW-leeftijd THEN verwijzing AIO (SVB)

**Processtap 4.5 — Kostendelersnorm-check (art. 22a Pw) [M/H]**

Beslisregel:
- INPUT: aantal meerderjarige personen ≥ 27 jaar met hoofdverblijf op hetzelfde adres (uit BRP)
- LET A = totaal aantal kostendelers (meerderjarige bewoners ≥ 27 jaar, exclusief de partner van de aanvrager als beiden bijstand aanvragen als gehuwden)
- IF A ≥ 2 THEN kostendelersnorm is van toepassing
- Uitzonderingen op kostendelersnorm — de volgende personen tellen NIET mee als kostendeler: personen jonger dan 27 jaar, studenten die studiefinanciering ontvangen (art. 19a lid 1 sub c Pw), commerciële onderhuurders/kostgangers (met schriftelijk huurcontract en commerciële huur), personen die gedetineerd zijn, personen die in een inrichting verblijven

De BRP-registratie kan als eerste indicatie dienen, maar de feitelijke beoordeling van uitzonderingen (met name commerciële onderhuur en de vraag of iemand daadwerkelijk op het adres woont) is deels menselijk.

**Output stap 4:** leefvorm (ALLEENSTAANDE / ALLEENSTAANDE_OUDER / GEHUWDEN), leeftijdscategorie (18-20 / 21+), aantal kostendelers (A), kostendelersnorm ja/nee.

---

### B.5 STAP 5 — Vermogenstoets

**Wettelijke grondslag:** art. 34 Pw [PiB-F1 gewijzigd] (vermogensdefinitie en -grens), art. 34 lid 3 Pw (vermogensgrenzen), art. 31 lid 2 Pw (vrijgelaten middelen).

Per 01-01-2026 (PiB fase 1) is de vermogenstoets vereenvoudigd: de vermogensgrens geldt als doorlopende toets (niet meer apart vastgesteld bij aanvang bijstand), en schulden worden afgetrokken van bezittingen.

**Processtap 5.1 — Vermogen berekenen [M/H]**

FORMULE: netto_vermogen = bezittingen − schulden

Bezittingen omvatten: saldi op alle bankrekeningen (inclusief cryptovaluta, PayPal, creditcard), waarde beleggingen (aandelen, obligaties, fondsen), waarde kostbare bezittingen (auto, caravan, boot, juwelen), waarde eigen woning (WOZ-waarde minus hypotheekschuld = overwaarde). Schulden die in mindering worden gebracht: formele schulden (persoonlijke leningen, creditcard-schulden, belastingschulden) worden afgetrokken. Informele schulden (schulden aan familie of bekenden zonder schriftelijke overeenkomst) worden niet automatisch meegeteld — de beoordeling hiervan is menselijk (H).

Specifieke vrijstellingen die NIET als vermogen meetellen: uitvaartkostenreservering op geblokkeerde rekening tot €8.600 (gehuwden/alleenstaande ouder) of €4.300 (alleenstaande), spaarrekening die is afgesloten in het kader van het levensloopregeling, bezit dat noodzakelijk is voor de uitoefening van het beroep (bij zelfstandigen).

**Processtap 5.2 — Vergelijking met vermogensgrens [M]**

Vermogensgrenzen per 01-01-2026 (art. 34 lid 3 Pw):
- Alleenstaande: €8.000
- Alleenstaande ouder of gehuwden: €16.000

Beslisregel:
- IF netto_vermogen (exclusief eigen woning) ≤ vermogensgrens_voor_leefvorm THEN vermogenstoets OK → door naar stap 5.3 voor eigen woning
- IF netto_vermogen > vermogensgrens_voor_leefvorm THEN afwijzen: aanvrager moet eerst vermogen interen tot onder de grens

**Processtap 5.3 — Eigen woning (art. 34 lid 2 Pw) [M/H]**

Beslisregel:
- IF aanvrager heeft eigen woning THEN
  - overwaarde = WOZ_waarde − hypotheekschuld
  - IF overwaarde ≤ €67.500 THEN bijstand als gift (normale verlening)
  - IF overwaarde > €67.500 THEN bijstand wordt (deels) als lening verstrekt (krediethypotheek). De berekening van de krediethypotheek is machine-uitvoerbaar; de beslissing om al dan niet een krediethypotheek te vestigen kan menselijke tussenkomst vereisen.

**Output stap 5:** vermogenstoets GESLAAGD/NIET GESLAAGD, eventueel bijstand als lening bij overwaarde eigen woning > €67.500.

---

### B.6 STAP 6 — Normberekening

**Wettelijke grondslag:** art. 20-23 Pw (normen), art. 22a Pw (kostendelersnorm), art. 25-28 Pw (normafwijkingen), art. 19 lid 3 Pw (vakantietoeslag 5%).

Dit is het meest machine-uitvoerbare onderdeel van de gehele pijplijn. Alle berekeningen volgen vaste formules met landelijk vastgestelde parameters die halfjaarlijks worden gepubliceerd.

**Processtap 6.1 — Basisnorm selecteren [M]**

Normen per 01-01-2026 (netto per maand, inclusief 5% vakantietoeslag):

Leefvorm 21 jaar tot AOW-leeftijd:
- Alleenstaande (art. 21 sub a Pw): €1.401,50
- Alleenstaande ouder (art. 21 sub a Pw — zelfde norm als alleenstaande): €1.401,50
- Gehuwden, beide 21+ (art. 21 sub c Pw): €2.002,13

Leefvorm 18-20 jaar (jongerennorm, art. 20 Pw, gewijzigd PiB fase 1):
- Alleenstaande of alleenstaande ouder 18-20 jaar: €345,99
- Gehuwden, beide 18-20 jaar, zonder kinderen: €691,98
- Gehuwden, beide 18-20 jaar, met kinderen: €1.092,41
- Gehuwden, één partner 18-20, ander 21+, zonder kinderen: €1.347,06
- Gehuwden, één partner 18-20, ander 21+, met kinderen: (**verifiëren** — de combinatie hangt af van de precieze normopbouw per partner)

Aanvullende bijstand 18-20 jaar uitwonend (art. 20 lid 2 Pw, PiB fase 1 — nu als algemene bijstand):
- IF leeftijd 18-20 AND uitwonend AND ouders kunnen niet bijdragen THEN college kan norm verhogen met aanvullende bijstand. Per 01-01-2026 is dit niet langer bijzondere bijstand maar algemene bijstand. Het bedrag is maximaal het verschil tussen de jongerennorm en de norm 21+ (€1.401,50 − €345,99 = €1.055,51), maar de exacte hoogte wordt individueel vastgesteld op basis van de vraag of de ouders redelijkerwijs kunnen bijdragen. De beoordeling van de ouderbijdrage is menselijk (H).

**Processtap 6.2 — Kostendelersnorm berekenen (art. 22a Pw) [M]**

IF kostendelersnorm is van toepassing (stap 4.5: A ≥ 2 kostendelers) THEN:

FORMULE: kostendelersnorm = ((40 + A × 30) / (A × 100)) × gehuwdennorm

Uitgeschreven per 01-01-2026 (gehuwdennorm = €2.002,13):
- A = 2 kostendelers: ((40 + 60) / 200) × €2.002,13 = 0,50 × €2.002,13 = €1.001,07 per persoon
- A = 3 kostendelers: ((40 + 90) / 300) × €2.002,13 = 0,4333 × €2.002,13 = €867,59 per persoon
- A = 4 kostendelers: ((40 + 120) / 400) × €2.002,13 = 0,40 × €2.002,13 = €800,85 per persoon
- A = 5 kostendelers: ((40 + 150) / 500) × €2.002,13 = 0,38 × €2.002,13 = €760,81 per persoon

De kostendelersnorm kan niet lager worden dan de norm die geldt als de formule uitgewerkt het laagst mogelijke bedrag oplevert. In de praktijk is het minimum gelijk aan de formule met het werkelijke aantal kostendelers.

Per PiB fase 1: het college heeft per 01-01-2026 de bestaande bevoegdheid om in gerechtvaardigde omstandigheden specifieke personen na periodieke toetsing voor een bepaalde duur van de kostendelersnorm uit te zonderen (motie Ceder, door SZW bevestigd in gemeentenieuws). Dit is een kan-bepaling en vereist menselijke beoordeling (H).

**Processtap 6.3 — Normafwijkingen (art. 25-28 Pw) [M/H]**

Het college kan de norm in individuele gevallen aanpassen:
- Art. 25 Pw: verhoging norm in uitzonderlijke situaties (hogere bijzondere kosten die niet door bijzondere bijstand worden gedekt) [H]
- Art. 26 Pw: verlaging norm tot maximaal 20% bij ontbrekende woonlasten (bijv. daklozen) [M/H]
- Art. 27 Pw: verlaging norm bij lagere algemene noodzakelijke bestaanskosten (bijv. wonen bij ouders, wonen in inrichting) [H]
- Art. 28 Pw: verhoging norm voor alleenstaande ouders die geen aanspraak op kinderbijslag kunnen maken [H]

In de Rotterdamse praktijk wordt de verlaging van 20% bij ontbrekende woonlasten standaard toegepast voor daklozen; dit is een parametergestuurde beslissing die na classificatie machine-uitvoerbaar is.

**Processtap 6.4 — Toepasselijke norm vaststellen [M]**

RESULTAAT: toepasselijke_norm =
- IF geen kostendelersnorm: basisnorm_voor_leefvorm (stap 6.1) ± eventuele afwijkingen (stap 6.3)
- IF kostendelersnorm: kostendelersnorm_per_persoon (stap 6.2) ± eventuele afwijkingen (stap 6.3)

**Output stap 6:** toepasselijke bijstandsnorm per maand (inclusief 5% vakantietoeslag), met motivering welke normcategorie en eventuele afwijkingen zijn toegepast.

---

### B.7 STAP 7 — Middelentoets (inkomen)

**Wettelijke grondslag:** art. 19 lid 1-2 Pw (recht op bijstand indien middelen < norm; bijstand = norm − middelen), art. 31 Pw (begrip middelen), art. 32 Pw (begrip inkomen), art. 33 Pw (inkomen uit vermogen), art. 39 Pw [PiB-F1] (giftenvrijlating).

**Processtap 7.1 — Inkomen vaststellen [M/H]**

Het systeem berekent het totale inkomen op basis van Suwinet-gegevens en eigen opgave.

Als inkomen wordt aangemerkt (art. 32 Pw): inkomsten uit arbeid in loondienst (bruto minus loonheffing = netto; te verifiëren via UWV polisadministratie en loonstroken), inkomsten uit zelfstandig beroep of bedrijf, uitkeringen (WW, WIA, WAO, Anw, etc.), alimentatie ontvangen, overige inkomsten (huurinkomsten, rente, dividend), vakantiegeld voor zover bovenop de norm ontvangen.

NIET als inkomen aangemerkt (art. 31 lid 2 Pw, selectie relevante posten): kinderbijslag (art. 31 lid 2 sub c), kinderopvangtoeslag (art. 31 lid 2 sub d), zorgtoeslag (art. 31 lid 2 sub c), huurtoeslag (art. 31 lid 2 sub c), persoonsgebonden budget (art. 31 lid 2 sub g), schadevergoedingen (art. 31 lid 2 sub l/m), giften tot €1.200 per kalenderjaar (art. 39 Pw nieuw, PiB fase 1) — bijdragen van voedselbanken, kledingbanken, speelgoedbanken en Stichting Jarige Job tellen niet mee in de €1.200-grens (volledig vrijgesteld, amendement Ceder).

**Processtap 7.2 — Inkomensvrijlating bij werk [M]**

Voor bijstandsgerechtigden die reeds werken naast de bijstand gelden inkomensvrijlatingen (art. 31 lid 2 sub n-r Pw). Per 01-01-2026 (geldend recht): de gewone inkomensvrijlating bedraagt 25% van het inkomen uit arbeid, maximaal €253,- per maand (art. 31 lid 2 sub n Pw — het exacte bedrag per 01-01-2026 moet worden geverifieerd; dit betreft het bedrag dat jaarlijks wordt geïndexeerd). De vrijlating geldt maximaal 30 maanden achtereen. Voor alleenstaande ouders met een kind jonger dan 12 jaar geldt een aanvullende vrijlating van 12,5% bovenop de gewone vrijlating (art. 31 lid 2 sub r Pw). Voor medisch urenbeperkte personen geldt een vrijlating van 15% (art. 31 lid 2 sub q Pw).

Merk op: de inkomstenvrijlating via art. 34a Pw (automatische verrekening met vrijlating) is fase 2 (gedoogd in 2026, formeel per 01-01-2027). Rotterdam werkt feitelijk al met automatische verrekening via OIB-AIV, maar de wettelijke basis voor de nieuwe vrijlatingsstructuur is per 19-02-2026 nog niet van kracht.

**Processtap 7.3 — Berekening bijstandshoogte [M]**

FORMULE: bijstand_per_maand = toepasselijke_norm − (inkomen − eventuele_vrijlating)

Concreet:
- netto_inkomen_relevant = totaal_inkomen − vrijgestelde_inkomsten − inkomensvrijlating
- IF netto_inkomen_relevant < 0 THEN netto_inkomen_relevant = 0
- bijstand = toepasselijke_norm − netto_inkomen_relevant
- IF bijstand ≤ 0 THEN geen recht op bijstand (inkomen is gelijk aan of hoger dan de norm)
- IF bijstand > 0 THEN recht op bijstand ter hoogte van het berekende bedrag

Voorbeeld per 01-01-2026: alleenstaande 35 jaar, geen kostendelers, parttime inkomen uit arbeid van €600 netto per maand, geen vrijlating (nog niet eerder toegepast):
- toepasselijke_norm = €1.401,50
- inkomensvrijlating = 25% × €600 = €150 (mits < maximum)
- netto_inkomen_relevant = €600 − €150 = €450
- bijstand = €1.401,50 − €450 = €951,50 per maand

**Output stap 7:** bijstandshoogte per maand, specificatie van meegeteld inkomen en toegepaste vrijlatingen.

---

### B.8 STAP 8 — Toekenningsbesluit

**Wettelijke grondslag:** art. 43 lid 1 Pw (vaststelling recht), art. 44 lid 1 Pw (ingangsdatum), art. 44a Pw [PiB-F1] (plan van aanpak), art. 45 Pw (maandelijkse betaling), art. 9 Pw (verplichtingen).

**Processtap 8.1 — Beschikking opstellen [M/H]**

De toekenningsbeschikking bevat: de beslissing (toekenning of afwijzing met motivering), de ingangsdatum (meldingsdatum of, bij terugwerkende kracht, tot max. 3 maanden eerder), de leefvormcategorie, de toepasselijke norm per maand, de berekening van de bijstandshoogte (norm − inkomen), de verrekening met het voorschot, de aan de bijstand verbonden verplichtingen. Het opstellen van de beschikking is grotendeels machine-uitvoerbaar op basis van de uitkomsten van stappen 3-7. De motivering vereist in complexe gevallen menselijke aanvulling.

**Processtap 8.2 — Verplichtingen vastleggen [M/H]**

Aan de bijstand zijn de volgende verplichtingen verbonden die in de beschikking worden opgenomen:

Arbeidsverplichting (art. 9 lid 1 sub a Pw): de bijstandsgerechtigde is verplicht naar vermogen algemeen geaccepteerde arbeid te verkrijgen, te aanvaarden en te behouden. Het college kan op grond van art. 9 lid 2 Pw tijdelijk ontheffen (medische gronden, zorgtaken, dringende redenen). Per 01-01-2026 kan het college op grond van art. 6b Pw vaststellen dat iemand medisch urenbeperkt is. De beslissing over ontheffing is menselijk (H).

Inlichtingenplicht (art. 17 lid 1 Pw): de bijstandsgerechtigde moet alle feiten en omstandigheden melden die van invloed kunnen zijn op het recht op bijstand. Wijzigingen moeten binnen twee weken worden doorgegeven.

Medewerkingsplicht (art. 17 lid 2 Pw): de bijstandsgerechtigde verleent medewerking aan onderzoek naar het recht op bijstand.

Tegenprestatie (art. 9 lid 1 sub c Pw, Verordening tegenprestatie 2015, CVDR348721): per 19-02-2026 formeel nog van kracht. Per PiB fase 3 (verwacht 01-01-2027) vervangen door maatschappelijke participatie. Rotterdam mag vooruitlopen.

**Processtap 8.3 — Plan van aanpak (art. 44a Pw, PiB fase 1) [H]**

Nieuw per 01-01-2026: het college stelt samen met de bijstandsgerechtigde een plan van aanpak op. Dit plan beschrijft de afspraken over de ondersteuning die de bijstandsgerechtigde ontvangt en de verplichtingen die daarbij horen. Het plan wordt periodiek geëvalueerd en zo nodig bijgesteld. Dit is volledig menselijk — de werkcoach stelt het plan op in samenspraak met de cliënt, op basis van de intake en profilering.

**Output stap 8:** toekenningsbeschikking met alle parameters, verplichtingen vastgelegd, plan van aanpak opgesteld.

---

### B.9 STAP 9 — Eerste betaling en verrekening voorschot

**Wettelijke grondslag:** art. 45 Pw (betaling per maand), art. 52 lid 3 Pw (verrekening voorschot).

**Processtap 9.1 — Eerste uitkering berekenen en verrekenen [M]**

FORMULE:
- eerste_betaling = bijstand_per_maand − eventueel_reeds_betaald_voorschot_voor_dezelfde_periode
- IF eerste_betaling < 0 THEN terugvordering verschil (of verrekening in volgende maanden)
- Nabetaling achterstallige perioden (van ingangsdatum tot eerste reguliere betaling) geschiedt bij eerste betaling

**Processtap 9.2 — Betaalschema vaststellen [M]**

De bijstand wordt maandelijks betaald (art. 45 Pw). Het vakantiegeld (5% van de norm) wordt maandelijks gereserveerd en in mei/juni uitbetaald. Inkomstenverrekening geschiedt maandelijks via OIB-AIV (automatisch) of handmatig op basis van loonstroken (binnen 14 dagen aanleveren, verwerking binnen 5 werkdagen).

**Output stap 9:** eerste betaling uitgevoerd, eventuele nabetaling en voorschotverrekening verwerkt, maandelijks betaalschema ingericht.

---

### B.10 Parameteroverzicht per 01-01-2026

Alle harde parameters die nodig zijn voor de bovenstaande pipeline, samengevat in één tabel.

| Parameter | Waarde per 01-01-2026 | Bron | Wijzigingsfrequentie |
|-----------|----------------------|------|---------------------|
| Bijstandsnorm alleenstaande 21+ | €1.401,50 /mnd incl. VT | Rijksoverheid | Halfjaarlijks (jan/jul) |
| Bijstandsnorm gehuwden 21+ | €2.002,13 /mnd incl. VT | Rijksoverheid | Halfjaarlijks |
| Jongerennorm 18-20 alleenstaand | €345,99 /mnd incl. VT | Rijksoverheid | Halfjaarlijks |
| Jongerennorm 18-20 gehuwden zonder kinderen | €691,98 /mnd incl. VT | Rijksoverheid | Halfjaarlijks |
| Jongerennorm 18-20 gehuwden met kinderen | €1.092,41 /mnd incl. VT | Rijksoverheid | Halfjaarlijks |
| Gehuwden één 18-20 + één 21+ z.k. | €1.347,06 /mnd incl. VT | Rijksoverheid | Halfjaarlijks |
| Vermogensgrens alleenstaande | €8.000 | Art. 34 lid 3 Pw | Jaarlijks (jan) |
| Vermogensgrens gehuwden/all. ouder | €16.000 | Art. 34 lid 3 Pw | Jaarlijks (jan) |
| Overwaardegrens eigen woning | €67.500 | Art. 34 lid 2 Pw | Jaarlijks (jan) |
| Uitvaartkostenreservering all. | €4.300 | Art. 34 lid 2 sub d Pw | Jaarlijks (jan) |
| Uitvaartkostenreservering gezin | €8.600 | Art. 34 lid 2 sub d Pw | Jaarlijks (jan) |
| Giftenvrijlating per kalenderjaar | €1.200 | Art. 39 Pw (PiB-F1) | Onbekend (nieuw) |
| Voorschot minimumpercentage | 95% | Art. 52 Pw (PiB-F1) | Wettelijk vast |
| Kostendelersnorm-formule | ((40+A×30)/(A×100)) × gehuwdennorm | Art. 22a Pw | Vast; gehuwdennorm wijzigt halfjaarlijks |
| Vakantietoeslag | 5% van netto-norm | Art. 19 lid 3 Pw | Wettelijk vast |
| Beslistermijn aanvraag | 8 weken | Art. 4:13 Awb | Wettelijk vast |
| Voorschottermijn | 4 weken na aanvraag | Art. 52 lid 1 Pw | Wettelijk vast |
| Zoektermijn jongeren < 27 | 4 weken | Art. 41 lid 4 Pw | Wettelijk vast (kan-uitzondering PiB-F1) |
| Verkorte heraanvraag-termijn | 12 maanden | Art. 43a Pw (PiB-F1) | Wettelijk vast |
| Terugwerkende kracht max. | 3 maanden vóór meldingsdatum | Art. 44 lid 4 Pw (PiB-F1) | Wettelijk vast |
| Bruto referentieminimummaandloon | €2.294,40 | KB | Halfjaarlijks |
| Minimumuurloon | €14,71 | KB | Halfjaarlijks |

---

### B.11 Machine-uitvoeringsclassificatie samenvatting

| Stap | Beschrijving | Classificatie | Toelichting |
|------|-------------|---------------|-------------|
| 1.1 | Meldingsdatum registreren | **M** | Systeemregistratie |
| 1.2 | Zoektermijn-check leeftijd | **M** (leeftijdssplit) / **H** (uitzondering) | Leeftijd automatisch; kwetsbaarheidsuitzondering menselijk |
| 2.1 | Identificatie | **M/H** | Documentvalidatie automatisch; twijfelgevallen menselijk |
| 2.2 | Verkorte aanvraag-check | **M** | Eerdere bijstandshistorie automatisch opvraagbaar |
| 2.4 | Voorschot berekenen | **M** | 95% × verwachte norm |
| 3.1 | Rechthebbende-check | **M/H** | Nationaliteit/verblijfsrecht deels automatisch, deels complex |
| 3.2 | Uitsluitingsgronden | **M/H** | Meeste checks automatisch, sommige menselijk |
| 3.3 | Voorliggende voorziening | **H** | Vereist inhoudelijke beoordeling |
| 4.1 | Leefvormbepaling basis | **M/H** | BRP als indicatie, gezamenlijke huishouding menselijk |
| 4.2 | Zorgbehoefte-uitzondering | **H** | Volledig menselijke beoordeling |
| 4.5 | Kostendelersnorm-check | **M/H** | BRP-telling automatisch, uitzonderingen soms menselijk |
| 5.1 | Vermogen berekenen | **M/H** | Formele schulden automatisch, informele schulden menselijk |
| 5.2 | Vermogensgrens-vergelijking | **M** | Pure parametercheck |
| 5.3 | Eigen woning-toets | **M/H** | WOZ en hypotheek automatisch, krediethypothekeerbesluit soms menselijk |
| 6.1 | Basisnorm selecteren | **M** | Tabel-lookup op leefvorm en leeftijd |
| 6.2 | Kostendelersnorm berekenen | **M** | Vaste formule |
| 6.3 | Normafwijkingen | **H** | Individuele beoordeling |
| 7.1 | Inkomen vaststellen | **M/H** | Suwinet-gegevens automatisch, overige bronnen soms handmatig |
| 7.2 | Inkomensvrijlating | **M** | Vaste formule en parameters |
| 7.3 | Bijstandshoogte berekenen | **M** | Norm − inkomen |
| 8.1 | Beschikking opstellen | **M/H** | Template automatisch, motivering soms handmatig |
| 8.3 | Plan van aanpak | **H** | Volledig menselijk |
| 9.1 | Eerste betaling | **M** | Verrekening met voorschot |

Van de 23 processtappen zijn er 10 volledig machine-uitvoerbaar (M), 10 gedeeltelijk machine-uitvoerbaar (M/H), en 3 volledig menselijk (H). De kernberekening — norm, kostendelersnorm, middelentoets, bijstandshoogte — is volledig machine-uitvoerbaar. De menselijke beslismomenten concentreren zich rond de leefvormbepaling (gezamenlijke huishouding, zorgbehoefte), de voorliggende-voorzieningencheck, normafwijkingen, en het plan van aanpak.
