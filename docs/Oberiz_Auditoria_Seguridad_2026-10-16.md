# Auditoría de seguridad previa a 1.0.4 — 2026-10-16

## Cierre de la revisión posterior — correcciones autorizadas

Los cuatro hallazgos siguientes quedan corregidos para v1.0.4: los aliases
permitidos comparten credenciales mediante un cliente sin proxy ni redirecciones
automáticas y con la dirección pública validada fijada; el login valida cada
salto, con máximo cinco y sin puertos arbitrarios. El origen local configurado
explícitamente sigue permitido. Los metadatos se consumen secuencialmente, sin
precargar todas las temporadas ni crear una tarea por recurso. La cuota global
protege el último informe válido del alcance que se actualiza y se aplica solo
después de validar la configuración. Los rangos de episodios usan suma comprobada
y la expansión del inventario se limita a 500.000 episodios únicos.

Se añaden regresiones para conexiones fijadas con cookies/cabeceras, aliases de
login privados, puertos, desbordamiento y conservación de informes con caché llena.
La primera contraseña por red no se modifica. Esta auditoría no demuestra el
consumo real de memoria de qBittorrent: queda pendiente observarlo en uso real.
Las secciones anteriores al cierre se conservan como historial de la revisión.

Verificación final: 124 pruebas unitarias correctas, una ignorada; dos migraciones;
cinco suites frontend y compilación; formato y Clippy correctos. Consulta de
dependencias: 252 versiones Rust activas sin avisos OSV y cero vulnerabilidades npm.

## Revisión de cambios posteriores — hallazgos históricos corregidos

El usuario aporta nuevos cambios sin preparar encima de los 57 archivos ya
preparados y solicita revisión antes de subir. Se han revisado sin modificar el
código. No se ha creado commit, etiqueta ni publicación. Los siguientes hallazgos
sustituyen el cierre anterior en lo que afecta a los archivos modificados después.

1. **P1 — Política de credenciales/DNS:** en `cardigann.rs`, la nueva rama
   `same_site` de `authorize_fetch` comprueba una resolución pública y después
   devuelve el cliente autenticado original, que vuelve a resolver al conectar.
   Reintroduce DNS rebinding para variantes www y subdominios. La política de
   redirección de login también admite la variante www sin validar sus IP.
   Hace falta conservar compatibilidad con los hosts legítimos pero fijar la IP
   validada y el origen/puerto autorizado antes de mandar cookies o cabeceras.
2. **P1 — Memoria de informes:** `metadata_many` crea una tarea por ruta y conserva
   todos los resultados. `calculate` precarga todas las temporadas en un BTreeMap,
   antes de aplicar el límite de 20.000 faltantes. Los máximos de entrada permiten
   hasta 1.000 series por 200 temporadas. Cuatro peticiones simultáneas no limitan
   la memoria retenida por todas sus respuestas. Procesar por lotes pequeños y
   liberar cada resultado, con cuota total y parada anticipada. No se lanzó una
   carga masiva contra proveedores ni una prueba de agotamiento de memoria.
3. **P2 — Evicción del último informe válido:** el DELETE global por cuota no
   excluye el informe válido del alcance que se está refrescando. Puede borrarlo
   antes de validar la conexión o antes de que termine su reemplazo. Reproducido
   en SQLite en memoria con la consulta actual y 200 informes: desaparece el único
   informe válido del alcance actual. Proteger ese informe hasta tener reemplazo;
   la expulsión de cachés antiguas de otros alcances puede conservarse.
4. **P2 — Desbordamiento en rangos:** `(episode..=episode + 50)` desborda con
   `episode_number=i64::MAX` y `episode_number_end` presente. La sonda aislada
   reproduce panic en depuración; en release la suma puede envolver y validar
   incorrectamente. Usar suma comprobada/saturada y un límite de numeración de
   protocolo; un inventario defectuoso no debe dejar un trabajo en running.

Verificación independiente de estos cambios: 120 pruebas unitarias correctas,
una omitida; formato y Clippy estrictos correctos; las cinco suites frontend y
compilación correctas. No se repitieron migraciones en este pase para no intentar
sobrescribir el ejecutable de desarrollo en marcha. Las pruebas actuales no
cubren los cuatro escenarios anteriores; que pasen no cierra los hallazgos.
Catálogo con IDs mayúsculos, omisión de entradas inválidas, temporales únicos y
aviso de omitidos: cambios coherentes con la compatibilidad y límites revisados.
La implementación de sesiones reutilizadas de qBittorrent no ha cambiado en este
diff posterior. Queda pendiente corregir los cuatro puntos y repetir verificación
antes de publicar 1.0.4. La primera contraseña por red sigue fuera de los cambios.

Estado posterior: el usuario autoriza publicar la versión corregida 1.0.4.
Las menciones a publicación detenida de este informe documentan la secuencia de
auditoría; no implican despliegue del servidor ni validación del RSS real de qBittorrent.

## Correcciones posteriores solicitadas por el usuario — local, sin publicar

Los hallazgos de abajo describen el estado ANTERIOR. Tras autorización se han
corregido SEC-01 a SEC-08 y el saneado adicional de mensajes. La primera contraseña
por red se conserva expresamente: no se ha reintroducido el requisito de loopback.

- SEC-01/02: cliente de descargas públicas sin proxy ambiental ni redirecciones
  automáticas, destino fijado tras validar DNS, IPv4-mapped normalizada y rangos
  especiales rechazados. Solo el origen configurado puede reutilizar credenciales
  o ser privado; un subdominio no hereda esa excepción. Login limitado a cinco
  saltos del mismo origen y formularios sin envío de credenciales a otro origen.
- SEC-03: restaurar conserva settings de seguridad/conexiones y `indexer_configs`
  actuales. No restaura sesiones. Confirmación traducida en seis idiomas actualizada.
- SEC-04: carga en streaming al disco privado, un upload, máximo 512 MiB, plazo
  de cinco minutos, cabecera SQLite validada y limpieza de temporales en error.
  Límite general de JSON de 2 MiB; credenciales 4 KiB e inventario 128 MiB mantienen
  sus límites específicos. Las copias exportadas siguen conteniendo secretos y
  deben custodiarse como tales; no se ha añadido cifrado con una clave local ficticia.
- SEC-05: limpieza de informes previos a tres terminales por alcance al solicitar
  refresco, preservando los válidos frente a errores. Hasta 200 informes, admisión
  limitada por 64 MiB ya almacenados, máximo 16 MiB por resultado y dos trabajos
  activos. Esto permite hasta dos resultados en curso sobre la cuota de admisión,
  no promete una cota exacta de 64 MiB en disco. Hasta 128 claves por trabajo activo;
  claves terminales caducan a siete días. Al alcanzar el cupo de alcances se rechaza
  trabajo nuevo, no se elimina silenciosamente el último informe válido.
- SEC-06: entradas de login expiradas eliminadas y registro máximo de 4096
  identidades; conserva el bloqueo creciente y agrupación IPv6 /64.
- SEC-07: catálogo con máximo 32 MiB comprimido, 128 MiB expandido, 20.000 entradas
  y 512 KiB por definición; YAML validado, enlace de destino rechazado y reemplazo
  mediante temporal. Una sincronización simultánea.
- SEC-08: `reg.exe`/`sc.exe` con ruta explícita System32, incluido tray. No se
  modifican los derechos del servicio instalado ni su cuenta.
- Secretos en URLs: limpieza al registrar y devolver historial/diagnósticos,
  incluyendo mensajes antiguos, userinfo y parámetros sensibles conocidos.
- Clippy corregido moviendo el módulo de pruebas de tema al final.
- CI con permisos mínimos por trabajo; adjuntos excluidos de Git y Docker.
  Datos privados Unix pasan por el mismo guard de rutas/enlaces que configuración.

### qBittorrent: causa encontrada y corrección

Oberiz recreaba el cliente HTTP/cookie en cada operación. Con autenticación por
cookie, cada sondeo podía iniciar otra sesión WebUI sin reutilizar la anterior;
incluso un bypass de autenticación puede crear sesiones si se descartan cookies.
Este patrón está documentado en la incidencia oficial:
https://github.com/qbittorrent/qBittorrent/issues/24695

Ahora conserva un cliente/cookie por conexión (caché LRU máxima de ocho), agrupa
logins simultáneos mediante generación y mutex, renueva tras rechazo y aplica
espera de 30 segundos después de un login fallido. Al desalojar una conexión
inactiva intenta cerrar solo su sesión, con un segundo de plazo. No cierra
sesiones de otras aplicaciones. No cambia preferencias ni torrents de qBittorrent.

Consultas sobre torrents conocidos filtradas por hash, tanto espera tras añadir
como comprobación de categoría/etiquetas. Solo se toma una lista completa cuando
no se conoce la identidad. Caché compartida de ocho segundos conservada y sondeos
del panel/Descargas sin solapamiento; al salir de Descargas se aborta su petición.

La prueba simulada verifica 25 consultas con UN login y 12 consultas concurrentes
tras caducar con UNA renovación adicional. No es una medición del RSS real del NAS.
Las sesiones antiguas ya creadas pueden seguir consumiendo memoria hasta caducar;
no se reinicia ni se cambia qBittorrent para forzar su liberación.

### Límites del cierre

Verificación posterior: 117 pruebas unitarias de backend aprobadas, una opcional
omitida; dos pruebas de migración aprobadas; Clippy estricto correcto. Las cinco
pruebas de frontend (idiomas, filtros, almacenamiento, importaciones e informes)
y la compilación pasan. Arranque local aislado 1.0.4: salud/base de datos `ok` y
`/api/settings` devuelve 401 sin sesión. La vista previa se actualiza en 2033;
no es un despliegue del servidor real ni una medición del consumo de qBittorrent.

No se han cambiado automáticamente TLS, proxy, cortafuegos, cuenta del servicio,
permisos efectivos de NAS o GitHub. Claves separadas por permiso, firma de
artefactos y fijación de herramientas auxiliares siguen siendo mejoras de diseño,
no vulnerabilidades explotadas demostradas. El estado real de memoria requiere
observación posterior con la instancia corregida. Publicación detenida.

## Dictamen y alcance

Mantener la publicación detenida. Se identifican dos hallazgos de prioridad alta
en las conexiones salientes y seis riesgos de prioridad media. No se han aplicado
correcciones ni publicado commits, etiquetas, paquetes o releases.

Revisión del código local de Oberiz: autenticación, rutas internas/públicas,
inventario e informes Cinetta, qBittorrent, archivos, copias, permisos,
dependencias, interfaz y distribución. Incluye pruebas automáticas y una sonda
aislada de la función real de filtrado IP. No constituye certificación de ausencia
de vulnerabilidades ni un pentest de infraestructura.

No se atacaron servicios reales ni se enviaron operaciones a Cinetta, NAS o
qBittorrent reales. No se inspeccionaron contraseñas ni contenido de bases de
datos privadas. No se verificaron ACL efectivas de la instalación, cortafuegos,
proxy, TLS, NAS, Docker publicado, permisos reales de GitHub o artefactos remotos.
La rama conserva los cambios de preparación de 1.0.4 anteriores a esta auditoría.

## Pruebas realizadas

- Backend: `cargo test --manifest-path backend/Cargo.toml --locked --bin oberiz`:
  **108 aprobadas, 0 fallidas, 1 omitida**. Se repitió fuera del aislamiento de red
  porque el primer intento bloqueaba conexiones HTTP a fixtures de loopback y
  operaciones sobre carpetas temporales. No se detuvo la instancia local.
- Frontend: `test:i18n`, `test:filters`, `test:storage`, `test:imports`,
  `test:missing` y compilación: correctos.
- Dependencias: OSV consultado sobre **252 versiones activas Rust**, sin avisos;
  `npm audit`, sin vulnerabilidades conocidas. Esto no cubre paquetes del sistema
  operativo, imágenes finales, herramientas descargadas por CI ni fallos lógicos.
- Formato Rust: correcto.
- Clippy con todos los destinos/características y advertencias como errores:
  **fallido** por `items_after_test_module` en `backend/src/db.rs:120`.
- Búsqueda limitada de patrones de claves privadas, tokens GitHub y claves AWS
  en 750 archivos versionados: sin coincidencias. No equivale a un escáner
  universal de secretos ni a revisión completa del historial. Adjuntos locales
  no versionados no se deben añadir a la publicación.
- No se repitieron los tests de integración de migración en este pase: construir
  todos los binarios podría intentar sobrescribir el ejecutable local en uso.

## Hallazgos altos

### SEC-01 — Redirecciones automáticas eluden la comprobación de destino

Evidencia: `backend/src/cardigann.rs:1026`, `authorize_fetch`, y
`follow_redirect_once` en la línea 1041.

Para un host no relacionado se crea un cliente con dirección DNS fijada, pero
sin `redirect(Policy::none())`. Reqwest 0.12.28 sigue redirecciones por defecto.
Un servidor público al que apunte un resultado de un indexador puede redirigir
a un servicio interno. Esa conexión sucede antes de que la aplicación reciba
la respuesta y vuelva a validar su destino. La fijación DNS inicial no protege
el siguiente host. El impacto concreto depende de los servicios internos
accesibles y de lo que admitan mediante GET; no se demostró escritura o ejecución
remota y no se probó contra infraestructura real.

Corrección: desactivar redirecciones automáticas en TODOS los clientes de esta
ruta; validar y fijar dirección en cada salto permitido. Probar una cadena
servidor público simulado → redirección a loopback, sin alcanzar el destino.
Considerar proxies ambientales: la política debe seguir siendo válida si se usan.

Fuente de la semántica por defecto:
https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.redirect

### SEC-02 — IPv4 representada como IPv6 supera el filtro de IP pública

Evidencia: `backend/src/cardigann.rs:970`, `is_public_ip`.

Se compiló en un ejecutable temporal la función extraída del código actual:

```
127.0.0.1                 allowed=false
::1                       allowed=false
::ffff:127.0.0.1           allowed=true
::ffff:192.168.1.1         allowed=true
```

El clasificador permite estas dos últimas aunque representan destinos IPv4
locales/privados. Es una reproducción del fallo de clasificación; la capacidad
de completar una conexión depende de la plataforma y conectividad IPv6/IPv4.
También falta una política completa para otros rangos especiales.

Corrección: normalizar IPv4-mapped y aplicar el filtro IPv4, además de rechazar
rangos no globales relevantes con pruebas. Revisar la excepción de hosts
relacionados en `authorize_fetch`: actualmente retorna antes de comprobar DNS;
confiar en un subdominio del tracker no demuestra que su IP sea pública. Mantener
una excepción explícita para indexadores locales configurados, no una autorización
implícita para cualquier subdominio de un indexador remoto.

## Hallazgos medios

### SEC-03 — Las copias restauran contraseñas y claves antiguas

Evidencia: `backend/src/backups.rs:338-352`.

Se excluye correctamente `auth_sessions`, pero se restaura toda la tabla
`settings`: vuelve el hash de contraseña anterior, las claves de integración y
otras credenciales. Una copia anterior a la configuración de contraseña puede
dejar el servicio nuevamente en configuración inicial remota. Requiere restauración
por un administrador: no es un acceso anónimo directo. Las copias también contienen
credenciales de terceros en claro y deben tratarse como secretos.

Corrección propuesta: conservar por defecto la identidad de seguridad actual y
pedir consentimiento separado para restaurarla; confirmar contraseña actual para
restaurar y verificar que nunca se abre accidentalmente un bootstrap remoto.
No cambiar silenciosamente la opción de configuración inicial remota aprobada.

### SEC-04 — Cargas de copias en memoria sin límite de concurrencia

Evidencia: `backend/src/routes/settings.rs:33`, `backups.rs:197` y
`main.rs`, límite global de cuerpos.

La ruta de subida admite 1 GiB y extrae `Bytes` completo; múltiples peticiones
administrativas pueden mantener varios cuerpos en RAM. El límite general es
512 MiB, demasiado amplio para gran parte de los formularios. La autenticación
reduce exposición pero no limita consumo por un cliente válido o defectuoso.

Corrección: límites específicos pequeños salvo copias, subida en streaming con
cuota/tamaño configurable y semáforo que se adquiera antes de leer el cuerpo.
Aplicar capacidad y plazos de lectura a peticiones internas costosas.

### SEC-05 — Informes e idempotencia crecen sin retención

Evidencia: `backend/src/missing.rs:195-239`, migración 0023.

Los dos trabajos simultáneos y límites por informe son útiles, pero no existe
cuota total ni limpieza por antigüedad. Cada refresco completado puede guardar
otro informe grande. Incluso con un trabajo activo, claves de idempotencia
distintas añaden registros al reutilizarlo. Un poseedor de la clave API, cuando
el permiso adicional está activo, puede hacer crecer la base y sus copias.

Corrección: retención por alcance, cuota global, límite de acciones por trabajo y
caducidad documentada de claves de idempotencia. Nunca borrar el último informe
válido únicamente porque falle el nuevo.

### SEC-06 — El registro de intentos de login no expira sus entradas

Evidencia: `backend/src/auth.rs:218-248`.

La espera por IP y el semáforo Argon2 existen. Sin embargo, cada identidad fallida
permanece en el HashMap hasta un login correcto de esa identidad; el tiempo de
reinicio reinicia contadores, pero no elimina entradas. Ataques desde muchos
orígenes pueden aumentar memoria durante toda la vida del proceso. Tras un proxy
de confianza deben validarse también las identidades reenviadas.

Corrección: caché acotada con TTL y limpieza; límite global de solicitudes de
autenticación antes de encolar trabajo caro. Preservar agrupación IPv6 /64.

### SEC-07 — Sin cuotas en descarga y extracción del catálogo de indexadores

Evidencia: `backend/src/indexers.rs:483` y `extract_v11` en la línea 1082.

El ZIP se descarga con `bytes()` y las entradas se descomprimen con `read_to_end`
sin máximos por archivo, acumulado o cantidad. El origen HTTPS es fijo y requiere
acción administrativa; no es una carga ZIP anónima. Un upstream comprometido o
un crecimiento descontrolado puede agotar RAM/disco. Usar solo el nombre final
evita el traversal clásico, pero no resuelve cuotas ni enlaces preexistentes.

Corrección: lectura acotada en streaming, límites comprimidos/descomprimidos,
validación de YAML antes de instalar, rechazo de enlaces y escritura temporal
atómica. Probar archivos sobredimensionados y ZIP de alta expansión.

### SEC-08 — Herramientas Windows se buscan por nombre

Evidencia: `backend/src/service.rs:210` (`reg`) y línea 302 (`sc`).

Instalar/configurar el servicio ejecuta herramientas sin ruta absoluta. Si un
directorio consultado por la búsqueda de ejecutables está bajo control de otro
usuario, puede ejecutarse un binario impostor con los permisos del instalador.
La explotación requiere control local de una ubicación de búsqueda y el arranque
de ese flujo; no se demostró una elevación real en esta máquina.

Corrección: rutas absolutas de System32, siguiendo el patrón ya utilizado para
PowerShell e icacls en la protección de almacenamiento. Revisar también tray y
scripts. La concesión de iniciar/detener a todos los usuarios autenticados es
otra decisión de disponibilidad que conviene limitar al grupo apropiado.

## Riesgos de despliegue y endurecimiento adicional

- **HTTP y cookies:** servidor HTTP y escucha 0.0.0.0 por defecto;
  `Secure` solo si se configura `OBERIZ_COOKIE_SECURE`. Fuera de loopback/red de
  confianza, usar HTTPS y cookies Secure. No activar Secure a ciegas en HTTP.
- **Primer administrador:** cualquier cliente con acceso puede establecer la
  primera contraseña mientras no exista. Es una decisión aprobada, no una
  regresión a corregir sin permiso. Preparar la instancia antes de exponerla;
  un token de instalación permitiría mantener configuración remota con más seguridad.
- **Permisos de API:** una misma clave pública permite operaciones distintas;
  habilitar informes no convierte esa clave en una clave exclusivamente de lectura.
  Valorar claves separadas por integración y permisos, rotación y límites.
- **Errores:** revisar `cardigann.rs:903`, que conserva un error HTTP sin pasar
  por el saneador común, y diagnósticos que presentan el historial de errores.
  Una URL con credenciales en parámetros podría llegar a un informe de soporte.
- **Almacenamiento:** credenciales salientes en SQLite sin cifrado. ACL Windows
  y directorios privados son la barrera principal. El usuario de servicio debe
  ser dedicado; LocalService comparte identidad con otros servicios de Windows.
  En Unix la protección del directorio de datos usa chmod, no la misma comprobación
  de enlaces/rutas amplias del protector de configuración: revisar coherencia.
- **CI/suministro:** acciones y bases Docker fijadas, usuario Docker sin root,
  pruebas previas a publicar y checksums son positivos. `cargo install
  cargo-cyclonedx` y herramientas de empaquetado no fijan versión; reducir permisos
  del token por job, fijar herramientas y estudiar firma de artefactos.
- **Contexto Docker:** no incluir adjuntos remotos, secretos, copias o archivos
  SQLite alternativos; revisar exclusiones antes de construir/publicar.
- **Sesiones/CSRF:** cookie HttpOnly/SameSite Strict y ausencia de CORS interno
  son defensas útiles. No se observó validación explícita Origin/Host: probar
  escenarios de mismo sitio distinto puerto y DNS rebinding antes de declarar
  cobertura completa. No se afirma un bypass CSRF demostrado.

## Protecciones comprobadas

- Contraseñas Argon2 con sal aleatoria, sesiones opacas aleatorias almacenadas
  como hash, expiración y revocación al cambiar contraseña; verificación de la
  contraseña actual para cambiarla. Cuerpo de credenciales limitado a 4 KiB.
- API interna protegida por sesión; API externa por clave, antes de JSON.
  Informes externos exigen además permiso explícito desactivado por defecto.
- Consultas parametrizadas en operaciones revisadas; identificadores dinámicos
  de restauración entrecomillados. No se identificó inyección SQL explotable.
- React sin usos encontrados de innerHTML/dangerouslySetInnerHTML/eval en el
  código de interfaz revisado; CSP, nosniff y bloqueo de enmarcado presentes.
- Informes con clientes sin redirecciones, tiempos máximos, respuesta acotada,
  inventario paginado estable y límites por trabajo; GET no crea trabajos.
- Informes no invocan operaciones de archivos ni qBittorrent. Escaneo local
  separado de operaciones, sin movimientos directos del NAS.
- Borrado manual de qBittorrent refleja la elección explícita de archivos;
  automatismos mantienen el bloqueo de borrado. Renombrado opt-in, validación
  de rutas relativas, comprobación de colisiones y recuperación detenida.
  Sigue existiendo la carrera inherente entre comprobar y renombrar frente a
  escritores externos: no prometer atomicidad que la API remota no ofrece.

## Bloqueo adicional de publicación

Clippy falla en `backend/src/db.rs:120` porque hay funciones después del módulo
de pruebas de tema. No es vulnerabilidad de ejecución, pero el workflow exige
Clippy sin advertencias y no debería publicar 1.0.4 así. Mover el módulo al final
y repetir las verificaciones, sin ocultar la advertencia.

## Orden propuesto, pendiente de autorización de implementación

1. SEC-01/02 y pruebas de regresión HTTP/IP que no cubre la batería actual.
2. SEC-03/04/05: seguridad de restauración y cuotas de recursos persistentes/RAM.
3. SEC-06/07/08: cotas, extracción y herramientas Windows.
4. Endurecimiento de despliegue consensuado, no cambios funcionales silenciosos.
5. Resolver Clippy, repetir pruebas y revisar el conjunto final antes de publicar.
