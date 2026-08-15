# Despliegue y publicación

## Artefacto inmutable

La promoción parte de un commit revisado. `main`, `production` y el tag anotado
deben resolver al mismo commit; no se recompone código entre etapas.

```mermaid
flowchart LR
    Feature["release/production-1.0.0"] --> PR["Pull request"]
    PR --> Matrix["CI Ubuntu + Windows"]
    Matrix --> Review["Revisión"]
    Review --> Main["main"]
    Main --> Production["production"]
    Production --> Tag["v1.0.0 anotado"]
    Tag --> Release["Production 1.0.0"]
    Main -. mismo SHA .-> Release
```

## Entorno

| Componente | Requisito                        |
| ---------- | -------------------------------- |
| Rust       | `>=1.96`, edición 2024           |
| Cargo      | `Cargo.lock` obligatorio         |
| Node.js    | `>=24`                           |
| npm        | `>=11`, instalación con `npm ci` |
| CI         | Ubuntu y Windows                 |
| permisos   | contenido en modo lectura        |

```bash
npm ci
npm run ci
git status --short
```

## Pipeline

```mermaid
sequenceDiagram
    participant Candidate as Candidato
    participant CI as GitHub Actions
    participant Main as main
    participant Prod as production
    participant Release as Release
    Candidate->>CI: push y PR
    CI->>CI: formato + sintaxis + clippy
    CI->>CI: build + 17 Rust + 33 Node
    CI->>CI: hashes + docs + banner
    CI-->>Candidate: satisfactorio
    Candidate->>Main: merge
    Main->>Prod: avanzar al mismo SHA
    Prod->>Release: tag anotado y publicación
    Release->>CI: verificar referencias finales
```

El workflow de integridad se ejecuta en `main`, `production`, tags y evento de
release. Cuando las referencias finales existen, resuelve el tag anotado y
compara los tres commits.

## Configuración externa

Una instancia conectada debe recibir parámetros mediante archivos montados o
variables gestionadas, y secretos desde un almacén autorizado. Se recomienda:

- identidad diferente por entorno y función;
- red de salida cerrada salvo destinos explícitos;
- límites de CPU, memoria y tiempo;
- logs estructurados con redacción;
- métricas de reservas, pendientes, exposición y cobertura;
- hash de configuración junto al commit ejecutado;
- backups cifrados y restauraciones ensayadas.

```mermaid
flowchart TB
    Artifact["Binario inmutable"] --> Runtime["Runtime"]
    Config["Configuración versionada"] --> Runtime
    Secrets["Gestor de secretos"] --> Runtime
    Runtime --> Metrics["Métricas"]
    Runtime --> Logs["Logs"]
    Runtime --> Journal["Journal persistente"]
    Metrics --> Monitor["Monitorización"]
    Logs --> Monitor
    Journal --> Reconcile["Reconciliación"]
```

## Reversión

La reversión selecciona una publicación anterior completa. Primero se detiene
admisión, se conserva el estado, se inventarían obligaciones pendientes y se
comprueba compatibilidad. Después se restaura el artefacto, se reproduce el
journal desde un punto conocido y se reabre con capacidad limitada.

## Criterios de aceptación

- [ ] suite local y árbol limpio;
- [ ] PR con ambos sistemas en verde;
- [ ] versiones Cargo y npm en `1.0.0`;
- [ ] documentación y banner verificados;
- [ ] cuatro blobs económicos preservados;
- [ ] `main`, `production` y tag alineados;
- [ ] release final no draft ni prerelease;
- [ ] integridad posterior satisfactoria.
