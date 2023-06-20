# TER_DATABASES
Aqui se guardan las bases de datos generales del TER, Son Creadas con el programa SavvyCAN u otros programas que permitan la edición de .dbcs
**La idea es que este repositorio sirva como unica fuente de verdad sobre las señales del coche, por favor acuerdate de hacer un commit cada vez que hagas un cambio.**

## Automatización
Todos los dbcs que se suban a este repositorio serán convertidos a sus correspondientes .h y .c para su uso
en los programas, mediante github actions.

# Inclusion como Submodulo
En tus proyectos de código es recomendable incluir este repositorio como submodulo para tener siempre la ultima versión del dbc a la hora de compilar, ejecutando
`$ git submodule add --name DBC https://github.com/Tecnun-eRacing/TER_DATABASES.git`,sobre la carpeta del proyecto de codigo, esto creara un repositorio dentro de este cada vez que vayas a compilar ejecuta un `$ git submodule update --remote` así contarás con la ultima versión del dbc siempre en tu código. No olvides incluir la carpeta DBC en tu IDE para que pueda encontrar los .h/.c.
