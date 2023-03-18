## Määrittelydokumentti

- Toteutan projektin Rust-ohjelmointikielellä.
  - Voin vertaisarvioida projekteja kielillä: Java, Python, Lua, C, C++, JavaScript, TypeScript, Go, C#.
- Ratkaistava ongelmani on miinaharavan pelaaminen optimaalisesti, ja toteutan algoritmin, joka ratkaisee kentät mallintamalla ne rajoitelaskentaongelmana, käyttäen David Becerran artikkelissa [0] mainittua Coupled Subsets CSP algoritmia. Tämä algoritmi tutkii yhtälöryhmiä, jotka luodaan perustuen miinaharava-kentän tilaan. Yhtälöryhmien tietorakenne on lista, joka koostuu yhtälöistä. Yhtälöt kuvataan parina, joka koostuu listasta muuttujia (A, B, C...), sekä odotusarvosta (N), jotka muodostavat yhtälön `A + B + C ... = N`.
- Ohjelma saa syötteenä argumentteja joita voi listata komentorivillä, jotka voi esimerkiksi määritellä kuinka monta peliä pelataan, ja minkä vaikeustasoisia kenttiä se luo pelattavaksi. Ohjelmalla tulee olemaan myös käyttöliittymä, joka näyttää visuaalisesti pelien etenemistä, jolla voi tutkia miten tekoäly pelaa miinaharavaa. Aikataulusta riippuen käyttöliittymästä voi tulla laajempi tai suppeampi.
- Optimaalista aika- ja tilavaativuutta on vaikea määritellä, koska miinaharavan pelaaminen optimaalisesti on NP-täydellinen ongelma. Voittotodennäköisyyksiä voi kuitenkin optimoida ja mitata, joten tavoitteena olisi päästä Becerran artikkelin [0] mainitsemiin noin 90%, 75% ja 30% voittotodennäköisyyksiin eri vaikeustasoisissa kentissä (beginner, intermediate ja expert).
  - Beginner-vaikeustason kentät ovat kokoa 8x8, 9x9 tai 10x10, ja niissä on 10 miinaa,
  - Intermediate-vaikeustason kentät ovat eri kokoisia 13x15 ja 16x16 välillä, ja niissä on 40 miinaa.
  - Expert-vaikeustason kentät ovat kokoa 16x30 tai 30x16, ja niissä on 99 miinaa.
- Lähteet:
  - [0] <https://dash.harvard.edu/bitstream/handle/1/14398552/BECERRA-SENIORTHESIS-2015.pdf>
  - [1] <https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf>
- Opinto-ohjelma: tietojenkäsittelytieteen kandidaatti (TKT)
- Projektin dokumentaatiossa (ja muualla) käytetty kieli tulee olemaan englanti.
