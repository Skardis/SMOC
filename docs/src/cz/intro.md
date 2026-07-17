# Co je to SMOC?

**Skardis Math On Crack (SMOC)** je vysoce výkonný, matematický "symbolický engine" napsaný v jazyce Rust. 
Na rozdíl od obyčejné kalkulačky, která všechna čísla převádí na zrádné datové typy `float` (což vede k zaokrouhlovacím chybám), SMOC zpracovává matematiku jako **abstraktní stromy** (AST). 

Zlomek $\frac{10}{3}$ v něm neznamená `3.33333333`, ale navždy zůstává strukturou `Fraction(10, 3)`, dokud s ním není algebraicky manipulováno dál. To garantuje 100% přesnost i pro ty nejsložitější algebraické rovnice a výpočty.

## Hlavní Vize
1. **Rozšiřitelnost:** Každý matematický vzorec je v SMOC jen "plugin" (Pravidlo). Kdokoliv může přidat nové pravidlo pro derivace nebo matice, a Engine si s tím poradí.
2. **Přesnost (Absolutní):** Celočíselné zlomky, parsování desetinných čísel na zlomky a striktní izolace od floatových výpočtů.
3. **Výkon a Elegance:** Rust zaručuje extrémní paměťovou bezpečnost, masivní rychlost (díky BFS vyhledávači stavů) a eleganci na backendu.

## Proč "On Crack"?
Tento engine se nezastaví před ničím. Převezme brutální Latexové zápisy, vygeneruje tisíce prvočísel pomocí bleskových keší (`smoc_prime`), provede binomické rozvoje obrovských závorek za pomoci Pascalova trojúhelníku (`smoc_pascaltriangle`) a vyplivne zkrácený, plně pokrácený matematický výsledek dříve, než mrknete okem.
