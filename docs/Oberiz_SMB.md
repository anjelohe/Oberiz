# Conexiones SMB del servidor de Oberiz

Incluido en v1.0.0 para instalaciones nativas de Windows. No depende de la marca del NAS, Cinetta ni de unidades de red conectadas en otra sesión de Windows.

## Uso opcional y compatibilidad

Sin añadir conexiones SMB, Oberiz no solicita credenciales del NAS, no establece conexiones nuevas ni cambia cuentas, permisos o sesiones existentes. Sigue utilizando las rutas locales, UNC y montajes a los que ya tenga acceso su proceso, como antes. Por tanto, una unidad visible para el usuario pero no para LocalSystem seguirá sin estar accesible al servicio hasta configurar acceso adecuado.

El selector abre la vista de discos y recursos en lugar del perfil `systemprofile`, incluso sin configurar SMB. No traslada ni borra archivos ni modifica destinos de importación, categorías de qBittorrent o correspondencias ya guardadas. Publicar una versión no actualiza automáticamente las instalaciones existentes.

El requisito de una cuenta de solo lectura se retiró por decisión del usuario. Se utilizarán los permisos reales de la cuenta configurada; el escaneo sigue sin escribir en los archivos de la biblioteca, mientras que mover/importar requiere los permisos correspondientes sobre origen y destino.

En Settings → Carpetas de la biblioteca → Servidores SMB / CIFS, introducir el nombre/IP del servidor, usuario y contraseña del NAS y pulsar **Conectar y guardar**. Usar HTTPS al configurar desde otro equipo. La contraseña nunca se devuelve por API; Windows DPAPI la cifra para esta máquina, permitiendo restaurarla desde la cuenta del servicio. La base de datos y las copias de seguridad siguen siendo privadas. Tras trasladar la configuración a otra máquina Windows se deben introducir nuevamente las credenciales.

Windows conecta sin asignar letras de unidad y enumera los recursos compartidos de disco. Si el servidor impide enumerarlos, se puede indicar un nombre de recurso conocido; los recursos añadidos se conservan en el listado. Conexiones con credenciales incompatibles producen un error explícito: Oberiz no desconecta otras sesiones de Windows para resolverlo.

El selector abre «Todos los discos y recursos»: unidades locales y recursos SMB guardados. Permite explorar rutas UNC, seleccionar varias carpetas y mantener ubicaciones guardadas cuando no responden; al intentar abrirlas muestra el error. Solo aparecen discos del NAS que estén compartidos por SMB, no dispositivos físicos que el NAS no exporte. Se respetan las restricciones `OBERIZ_FS_ALLOWED_ROOTS`.

Al iniciar la aplicación o el servicio se restablecen las conexiones guardadas en su propio contexto de Windows. Un NAS desconectado no bloquea indefinidamente el arranque: la restauración continúa en segundo plano después de diez segundos. Exploración, escaneo e importación utilizan rutas UNC y los permisos de esa cuenta SMB. No se cambian los permisos del NAS, ni se habilita SMB1/CIFS heredado; se utiliza la negociación del cliente SMB de Windows.

«Olvidar conexión guardada» elimina sus credenciales y recursos de la configuración de Oberiz, pero no fuerza la desconexión de sesiones Windows existentes. Las correspondencias de rutas siguen siendo necesarias si qBittorrent comunica rutas distintas, por ejemplo rutas Linux.

En Linux/Docker se utilizan recursos SMB montados por el sistema operativo y accesibles dentro del contenedor. El formulario de credenciales nativo de esta implementación no monta recursos Linux ni guarda contraseñas sin cifrar como alternativa.

Verificado: compilación de frontend, seis idiomas, validación de servidores, cifrado DPAPI y detección de datos corruptos, respuesta API sin contraseñas ni ciphertext, listado de recursos guardados sin intentar acceder al NAS y eliminación de credenciales. La autenticación real con la cuenta del NAS desde el servicio requiere introducir las credenciales en la nueva sección; no se han recuperado de otras aplicaciones ni configurado automáticamente en la instalación existente.
