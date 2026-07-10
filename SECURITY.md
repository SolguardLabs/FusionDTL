# Politica de Seguridad

Fusion DTL aplica una separacion estricta entre autorizacion, contabilidad,
liquidacion y observabilidad. El objetivo es que cada transicion relevante sea
verificable, reproducible y auditable a partir del journal canonico del libro.

## Alcance

Esta politica cubre:

- el binario Rust `fusion_dtl`;
- la logica del ledger y sus modulos de dominio;
- los escenarios de runtime;
- los tests Rust y JavaScript;
- los scripts de CI;
- la configuracion de GitHub Actions y Dependabot.

No cubre integraciones externas, infraestructura no incluida en este
repositorio ni despliegues personalizados.

## Controles Principales

El protocolo incorpora varios controles de defensa operativa:

- Firmas Ed25519 para ordenes de recibo y paquetes de liquidacion.
- Nonces independientes para recibos y paquetes.
- Digests canonicos para identidades, recibos, rutas, transacciones y estado.
- Validacion de roles antes de emitir o liquidar.
- Perfiles activos de participantes con vencimiento por epoca.
- Ventanas de liquidacion configurables.
- Politicas de capacidad por celda.
- Limites de riesgo por importe, comision y exposicion.
- Verificacion de conservacion por activo despues de transiciones economicas.
- Journal secuencial con digest de estado por entrada.

## Gestion de Dependencias

Las dependencias Rust se fijan con `Cargo.lock`. Las dependencias JavaScript se
fijan con `bun.lock`. Dependabot revisa semanalmente:

- crates de Cargo;
- paquetes gestionados por Bun;
- acciones de GitHub.

Los cambios de dependencias deben pasar el pipeline completo antes de
integrarse.

## Validacion Requerida

Antes de aceptar cambios se debe ejecutar:

```bash
bun run ci
```

Este comando cubre formato, lint, build y tests. Para validar solo la suite de
regresion:

```bash
bun run test:all
```

## Comunicacion de Incidencias

Los hallazgos sensibles deben comunicarse por un canal privado al equipo
mantenedor antes de publicarse. El reporte debe incluir:

- descripcion del comportamiento observado;
- pasos de reproduccion;
- impacto esperado sobre estado, saldos o permisos;
- version o commit analizado;
- salida relevante de comandos o tests.

No se deben abrir issues publicos con detalles tecnicos que permitan reproducir
un comportamiento no autorizado antes de que exista una revision coordinada.

## Criterios de Aceptacion

Un cambio relacionado con seguridad operacional debe:

- mantener la conservacion contable;
- preservar la determinacion de digests;
- incluir pruebas de regresion cuando cambie un flujo observable;
- no reducir las validaciones existentes sin justificacion tecnica;
- mantener verde el pipeline de CI.
