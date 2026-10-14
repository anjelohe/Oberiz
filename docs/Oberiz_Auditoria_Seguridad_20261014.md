# Auditoría de seguridad de Oberiz — 14 de octubre de 2026

## Estado después de las correcciones

Los hallazgos siguientes describen la auditoría original. Después se han añadido:

- Protección de directorios privados y sus archivos en Windows; permisos 0700
  para configuración/datos en Unix. Se conserva la identidad del propietario y
  del servicio. No se siguen enlaces/reparse points ni se cambian permisos del NAS.
- Primera contraseña solo desde loopback directo, sin proxies, y cambios de
  contraseña serializados para evitar la carrera de inicialización.
- Autenticación de API nativa/compatibilidad antes del cuerpo y un máximo de
  cuatro peticiones de integración concurrentes.
- rustls 0.23.46, source-map-js corregido; scraper actualizado sin fxhash y
  parser YAML mantenido sin serde_yaml/unsafe-libyaml. Verificado el catálogo.
- Auditoría OSV de dependencias activas y npm en CI/publicación.
- CSP y dirección de escucha configurable con OBERIZ_BIND_ADDRESS.
- LocalService para servicios Windows nuevos. En actualizaciones se conserva la
  cuenta existente: migrarla exige revisar permisos específicos de esa instalación.

Pendiente de instalación/verificación operativa: los cambios de código no
reemplazan por sí solos el ejecutable instalado. Las ACL se completaron tras la
elevación aceptada por el usuario: el helper devolvió `success` y la inspección
recursiva encontró cero reglas para Usuarios, Everyone o Authenticated Users en
datos y configuración, incluidos DB, WAL, SHM y copias. Se conservan las ACL anteriores.
No se han cambiado proxy, certificados, firewall, cuenta del servicio existente,
credenciales ni permisos de discos multimedia. Las conexiones siguen guardando
credenciales reversibles protegidas por permisos: no se ha cifrado íntegramente
SQLite ni se pueden proteger retroactivamente copias exportadas a otros destinos.

Ver [guía de seguridad](Oberiz_Seguridad.md) para instalación y límites.

## Alcance y resultado

Revisión del código local, incluidos cambios sin publicar respecto al commit
315711c: autenticación, API pública, biblioteca externa, credenciales, acceso a
archivos, operaciones qBittorrent, respaldos, empaquetado y dependencias.
Inspección de ACL de la instalación Windows sin leer ni imprimir credenciales.
Sin cambios de código/configuración/permisos, sin acciones sobre torrents ni NAS.
Este informe no constituye una certificación ni una prueba de penetración completa.

Hay riesgos que corregir antes de exponer directamente Oberiz a Internet.

## Hallazgos prioritarios

### SEC-01 — Alta: credenciales accesibles a otros usuarios locales

Confirmado: `%ProgramData%\Oberiz\data\oberiz.db` y el directorio de copias
tienen permisos heredados de lectura para `BUILTIN\Usuarios`. Los directorios
data/config/backups también presentan permisos de escritura heredados, cuyo
alcance efectivo debe revisarse al corregir la ACL.

`settings.rs:441-450` guarda las claves TMDB/TVDB y la contraseña qBittorrent
sin cifrado; `indexers.rs:762` guarda las credenciales del indexador en JSON.
La contraseña administrativa y la clave de API pública sí se almacenan como hashes.
Las copias SQLite contienen estos mismos datos sensibles.

La contraseña SMB usa DPAPI con `CRYPTPROTECT_LOCAL_MACHINE` (`smb.rs:251`).
No equivale a protección por usuario: otro usuario del mismo ordenador que
obtenga el blob puede descifrarlo. No se intentó descifrar ningún dato real.

Recomendación: restringir las ACL de datos, configuración, respaldos y archivos
existentes al servicio, administradores y al usuario propietario que corresponda.
Proteger también WAL/SHM; comprobar lectura/escritura desde una cuenta ordinaria.
Considerar cifrado de credenciales y copias, sin claves junto a los datos.

Referencia: [Microsoft, CryptProtectData](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata).

### SEC-02 — Alta condicionada: apropiación de la primera configuración

`auth.rs:347` permite `/api/auth/password` cuando todavía no hay contraseña.
`auth.rs:431` no exige un secreto de instalación ni que el cliente sea local.
`main.rs:278` escucha en `0.0.0.0:2032`.

En una instalación nueva expuesta a la red, quien llegue primero puede configurar
su propia contraseña y tomar el acceso administrativo. También hay una ventana
de carrera entre solicitudes iniciales, pues comprobar y guardar no es atómico.
No es un bypass de una contraseña ya configurada.

Recomendación: inicialización local o token de un solo uso, operación atómica y
restricción de escucha/firewall durante el primer arranque. No se comprobó el
estado de configuración del servicio instalado: no respondió la consulta local.

### SEC-03 — Media: consumo de memoria/CPU antes de autenticar la API

`routes/public.rs:31` admite cuerpos de biblioteca de hasta 128 MiB.
`external_library.rs:120` usa el extractor JSON antes de ejecutar `authorize`.
La ruta queda exceptuada de la sesión administrativa y depende de esa validación
en el handler. Un cliente sin clave puede provocar recepción/deserialización de
un cuerpo grande antes de ser rechazado, incluso si la API está desactivada.

Recomendación: autenticar en middleware/extractor previo al cuerpo, límites por
endpoint y concurrencia/rate limiting. Se confirmó el orden en código; no se hizo
una prueba de carga destructiva ni se midió el umbral de caída.

### SEC-04 — Dependencias: un aviso npm y siete avisos OSV en Cargo.lock

`npm audit` detectó `source-map-js 1.2.1`, transitivo de Vite/PostCSS, con aviso
alto de denegación de servicio al procesar mapas de fuentes. La corrección está
en 1.2.2. Su exposición principal aquí es la cadena de construcción: el frontend
publicado es estático, no un servicio que acepte mapas de fuentes de visitantes.
No se demostró explotación remota del Oberiz instalado.

Fuente: [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q).

Se contrastaron 330 paquetes/versiones de Cargo.lock con OSV:

- `rustls 0.23.45`: RUSTSEC-2026-0346/0347/0348/0349; corregidos en 0.23.46.
  Alcance condicionado a configuración: no se habilita Brotli en el árbol
  revisado, no se encontró uso explícito de ECH ni caché TLS personalizada con
  registro de secretos. El proveedor observado es ring; no se demostró que los
  cuatro problemas sean explotables con la configuración de Oberiz.
- `rsa 0.9.10`: RUSTSEC-2023-0071. Aparece en el lockfile, pero no en el árbol
  activo del objetivo Windows consultado. No se clasifica como vulnerabilidad
  activa demostrada de esta compilación.
- `unsafe-libyaml 0.2.11`: RUSTSEC-2024-0449, mantenimiento abandonado; llega
  mediante serde_yaml. Aviso informativo, no una explotación confirmada.
- `fxhash 0.2.1`: RUSTSEC-2025-0057, mantenimiento abandonado; aviso informativo.

Fuentes: [OSV](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0346),
[OSV](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0347),
[OSV](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0348),
[OSV](https://api.osv.dev/v1/vulns/RUSTSEC-2026-0349),
[RSA](https://rustsec.org/advisories/RUSTSEC-2023-0071.html),
[unsafe-libyaml](https://rustsec.org/advisories/RUSTSEC-2024-0449.html),
[fxhash](https://rustsec.org/advisories/RUSTSEC-2025-0057.html).

Recomendación: actualizar las versiones corregidas manteniendo pruebas/lockfiles;
planificar sustitución de dependencias abandonadas. Incorporar auditorías de
dependencias a CI; Clippy y las pruebas normales no consultan avisos de seguridad.
No se instaló cargo-audit: se utilizó OSV y se revisó el árbol activo con cargo tree.

## Endurecimiento adicional

- El servicio Windows se instala como LocalSystem (`service.rs:147`), lo que
  amplía las consecuencias de una intrusión. Valorar una identidad dedicada con
  permisos mínimos sin romper el acceso SMB ni la instalación existente.
- HTTP escucha en todas las interfaces. TLS debe terminar en un proxy, con
  `OBERIZ_COOKIE_SECURE=true`, firewall y proxies de confianza explícitos.
  No se verificaron el firewall, proxy ni exposición pública actuales.
- La clave de integración es común y sin scopes: quien la obtenga puede enviar
  informes y peticiones autorizadas, no solamente consultar. Separar permisos
  de biblioteca/peticiones y registrar cambios anómalos sería recomendable.
- Las rutas externas son información, no acceso a Cinetta. `superseded` no
  ejecuta borrados, pero no debe convertirse en borrado automático en Cinetta
  sin validación independiente, especialmente con archivos multiepisodio.

## Protecciones comprobadas

- Argon2 para contraseña, sesiones aleatorias de 256 bits almacenadas como hash,
  revocación y cookie HttpOnly/SameSite=Strict.
- Limitación de intentos de login y del trabajo Argon2 concurrente.
- API pública con hash de clave, desactivable e independiente de la sesión web.
- No se devuelven contraseñas SMB ni claves en las respuestas normales revisadas.
- Bloqueo del borrado de archivos qBittorrent en modo externo tanto en la ruta
  manual como en la implementación interna utilizada por rechazo/limpieza.
- Biblioteca Cinetta almacenada como rutas virtuales; el informe no abre ni
  modifica sus archivos. Protección de discos declarados offline y episodios dobles.
- Consultas parametrizadas y validación de nombres de respaldo y rutas; defensa
  de descargas de indexadores frente a destinos privados y redirecciones.
- Restaurar una copia no restaura sesiones antiguas revocadas.
- Actions y bases Docker fijadas; sin afirmar que eso sustituya actualizaciones.

## Verificación y límites

100 pruebas unitarias y una de migración pasan; la prueba optativa sobre discos
reales no se ejecutó. Sin lectura de secretos, envío de claves a terceros, cambios
de contraseña, explotación, pruebas de carga, borrado de torrents ni acceso al NAS.
Se inspeccionó la ACL real y los avisos vigentes; no la seguridad completa del
NAS/Cinetta, el firewall ni todas las plataformas/funcionalidades TLS posibles.

Orden sugerido: ACL/credenciales → primer arranque seguro → autenticar antes del
JSON → actualizaciones de dependencias → permisos mínimos y auditorías en CI.
