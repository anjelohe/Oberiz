# Seguridad y puesta en servicio

## Primera contraseña

La primera contraseña puede configurarse desde el navegador de otro dispositivo,
también mediante un proxy, sin acceder al escritorio del servidor. No se exige
una dirección de loopback. Completar este paso en una red de confianza antes de
publicar la instancia: mientras no hay contraseña, quien alcance el formulario
puede establecerla. Los cambios se serializan y, una vez configurada, sustituirla
requiere conocer la contraseña actual. Se mantienen los límites de longitud,
el hash seguro y la revocación de sesiones al cambiarla.

## Directorios privados

Datos, copias y configuración requieren permisos exclusivos del propietario,
administradores y cuenta de servicio. Windows aplica ACL a archivos existentes
y futuros; Linux utiliza directorios 0700. El helper solo debe apuntar a los
directorios privados de Oberiz, nunca a una raíz de disco, carpeta personal,
sistema operativo, enlace ni biblioteca multimedia.

La instalación existente en ProgramData requiere elevación de administrador
para actualizar permisos de archivos creados por el servicio. Las ACL anteriores
se han guardado localmente en `backend/backups/security-acl-20261014`.
El helper preparado es `backend/backups/complete-private-acl.ps1`; ejecutar con
Windows PowerShell como administrador si no aparece el aviso solicitado.
Su resultado queda en `elevated-result.txt`. Verificar ACL de DB, WAL, SHM,
respaldos y configuración antes de considerar cerrada la corrección.

Verificación realizada el 14 de octubre de 2026: helper `success`, sin reglas de
acceso de Usuarios, Everyone o Authenticated Users en los archivos y directorios
privados inspeccionados recursivamente. No se cambiaron permisos de bibliotecas.

## Cuenta y red

Servicios Windows nuevos usan LocalService, no LocalSystem. Una actualización
mantiene la cuenta existente para evitar romper rutas que dependan de sus permisos.
Cambiar esa cuenta requiere revisar primero el acceso necesario a los directorios.
SMB sigue usando las credenciales explícitas configuradas por el usuario.

`OBERIZ_BIND_ADDRESS=127.0.0.1:2032` limita la escucha a loopback cuando se usa un
proxy local. El valor por defecto conserva la escucha de red por compatibilidad.
Para acceso remoto: HTTPS, `OBERIZ_COOKIE_SECURE=true`, firewall y
`OBERIZ_TRUSTED_PROXIES` con únicamente las direcciones de proxies reales.
Estas opciones no instalan certificados ni modifican el firewall automáticamente.

## API y archivos

La clave se comprueba antes de deserializar cuerpos, incluso con la API desactivada.
La integración tiene límite de concurrencia. La clave actual conserva su contrato
para no romper Cinetta/Overseerr: no se ha cambiado ni sustituido automáticamente.
Debe mantenerse secreta; todavía no existen permisos separados por cliente/scope.
Las políticas automáticas y el rechazo de releases conservan los archivos.
En Descargas, el borrado manual pide confirmación y permite marcar «Borrar también
los archivos descargados», desmarcado por defecto. Al marcarlo, qBittorrent ejecuta
el borrado incluso si Oberiz no tiene acceso al NAS o usa inventario Cinetta.
Un informe Cinetta solo actualiza datos de Oberiz.
No utilizar `superseded` como instrucción de borrado sin validación independiente.

## Credenciales y copias

Las credenciales de conexiones son reversibles porque Oberiz debe autenticarse
con ellas. La base de datos y sus copias son confidenciales y se protegen mediante
ACL/permisos; no se ha añadido cifrado integral con una nueva clave de recuperación.
No publicar copias ni guardarlas en carpetas compartidas. Las copias antiguas o
exportadas fuera de los directorios protegidos necesitan protección en su destino.
Las contraseñas administrativas y claves de API se guardan como hash; SMB utiliza
DPAPI de máquina, que también depende de las ACL para aislar usuarios locales.

## Dependencias

Ejecutar `pwsh -File scripts/audit-dependencies.ps1` desde la copia de código.
Consulta OSV para versiones activas de Cargo (no entradas opcionales sin utilizar)
y npm para el frontend. La comprobación también forma parte de CI y la publicación.
No confundir un resultado limpio con una garantía frente a vulnerabilidades
desconocidas ni con una auditoría del NAS o de Cinetta.
