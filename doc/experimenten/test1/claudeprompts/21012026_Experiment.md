### Prompt

You are performing 'wetsanalyse' as a professional analist for turning laws into executable rules (check the work of Lokin for a more thorough explanation). Follow a five step plan as noted as analyseplan below. A hint: article 11 is very important.

please check out the CLAUDE.md file in regelrecht-mvp and follow instructions there. 
main task: your job is to create a machine interpretable version of the participatiewet (see the xml file in the repo starting with BWBR0015703 as a ground truth. 
Take into account the new norms as variables in the right articles. follow the schema from 'schema' and update the participatiewet.yaml in the folder 'regulation'. take into account the ontologies with concepts around income     
  from https://github.com/VNG-Realisatie/Ontologie-Inkomen/blob/main/ontologie-totaal.html and its links to other concepts in other repos.
  Use full text next to the analyzed text.
  Provide a log of the steps you took.
  Annotate where we need other laws to be formalized for it to be executable.
### analyseplan
1. area of interest: municipality who wants executable rules for making it easier and transparent for people to ask for allowances
2. **Doel**: Identificeren van de juridische 'grammatica'
3. Create a grammar and do the following:
	Gebruik van het juridisch analyseschema om formuleringen te classificeren
    - Toekennen van juridische klassen aan wetteksten
    - Identificeren van elementen zoals:
        - **Rechtssubjecten** - wie (partijen, personen)
        - **Rechtsobjecten** - waarover (zaken, goederen)
        - **Rechtsfeiten** - wat (gebeurtenissen, handelingen)
        - **Rechtsbetrekkingen** - rechten en plichten
        - **Voorwaarden** - wanneer van toepassing
        - **Rechtsgevolgen** - wat is het resultaat
4. - **Doel**: Expliciteren van interpretatie en betekenis
    - Definiëren van begrippen voor elke formulering
    - Bepalen van eigenschappen bij elk begrip
    - Maken van concrete voorbeelden die de betekenis verduidelijken
    - Vastleggen in kennismodellen (gegevensmodel, regelmodel, procesmodel)
    - Annoteren met metadata over interpretaties
5.  **Doel**: Toetsen van de analyse op juistheid
- **Activiteiten**:
    - Opstellen van juridische scenario's
    - Doorlopen van concrete voorbeelden
    - Testen van edge cases en grensgevallen
    - Multidisciplinaire review (juristen, ICT-ontwikkelaars, uitvoeringsexperten)
    - Iteratief verbeteren van de analyse
- **Output**: Gevalideerde en geteste analyseresultaten