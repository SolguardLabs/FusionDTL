# Integración

## CLI y SDK

El binario acepta un escenario y produce un objeto JSON por `stdout`. El SDK
Node.js proporciona una frontera estable para automatización local.

```mermaid
flowchart LR
    Service["Servicio"] --> SDK["FusionClient"]
    Scheduler["Proceso programado"] --> SDK
    SDK --> Spawn["spawnSync sin shell"]
    Spawn --> Cargo["cargo run --quiet"]
    Cargo --> Binary["fusion_dtl"]
    Binary --> JSON["ScenarioReport JSON"]
    JSON --> Validate["normalizeReport"]
    Validate --> Service
```

```js
const { FusionClient } = require("./sdk/client");

const client = new FusionClient({
    timeoutMs: 30_000,
    env: { RUST_BACKTRACE: "0" },
});

const report = client.run("snapshot");
const view = client.snapshot("issue");
```

## Contrato de informe

| Grupo          | Contenido                                         |
| -------------- | ------------------------------------------------- |
| raíz           | escenario, red, journal, digest y conservación    |
| `asset`        | ID, símbolo y decimales                           |
| `balances`     | saldos de las seis identidades del escenario      |
| `cells`        | reserva y obligación de core y edge               |
| `receipt`      | recibo completo o `null`                          |
| `transactions` | lista ordenada de `TxId`                          |
| `surface`      | contadores de registros, roles, lanes y políticas |

El normalizador exige enteros seguros no negativos y un digest hexadecimal de
32 bytes. Los campos económicos se conservan como enteros; para importes por
encima del rango seguro de JavaScript, una futura versión debe usar cadenas
decimales y versionar el esquema.

```mermaid
sequenceDiagram
    autonumber
    participant App as Aplicación
    participant SDK as FusionClient
    participant Process as Proceso Rust
    participant Validator as Validador
    App->>SDK: run("settle")
    SDK->>Process: spawn cargo
    Process-->>SDK: exit + stdout + stderr
    alt exit distinto de cero
        SDK-->>App: error estructurado
    else exit cero
        SDK->>Validator: parse + normalize
        Validator-->>SDK: report válido
        SDK-->>App: objeto validado
    end
```

## Errores y reintentos

Clasifica por separado:

- proceso ausente o entorno incompleto;
- tiempo máximo;
- salida no satisfactoria;
- JSON mal formado;
- esquema inválido;
- rechazo funcional de dominio.

Solo los fallos temporales del adaptador deben reintentarse automáticamente.
Un rechazo de firma, nonce, época, capacidad o riesgo requiere corregir la
entrada o esperar un cambio de estado.

```mermaid
flowchart TD
    Call["Solicitud"] --> Key["Clave idempotente"]
    Key --> Existing{¿Resultado previo?}
    Existing -- sí --> Return["Devolver resultado almacenado"]
    Existing -- no --> Execute["Ejecutar comando"]
    Execute --> Temporary{¿Fallo temporal?}
    Temporary -- sí --> Backoff["Espera exponencial acotada"]
    Backoff --> Execute
    Temporary -- no --> Persist["Persistir resultado o rechazo"]
    Persist --> Return
```

## Adaptador persistente

Una integración HTTP, RPC o cola debe autenticar en el borde, validar tamaños,
asignar claves de correlación y confirmar estado más evento atómicamente. No
debe introducir objetos de red, conexiones o credenciales dentro de los tipos
de dominio.

## Compatibilidad

`1.0.x` mantiene nombres, significado y unidades del informe. Añadir un campo
es compatible para consumidores tolerantes; retirar, renombrar o cambiar una
unidad exige una versión mayor.
