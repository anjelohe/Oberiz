import { spanish } from './spanish'

// Columns: English key | French | German | Italian | Portuguese (Portugal).
const rows = String.raw`
Library inventory|Inventaire de la bibliothèque|Bibliotheksinventar|Inventario della libreria|Inventário da biblioteca
qBittorrent does not provide atomic no-overwrite renaming. Visible destinations are checked; do not run other organizers on these folders simultaneously.|qBittorrent ne fournit pas de renommage atomique sans remplacement. Les destinations visibles sont vérifiées ; n’utilisez pas d’autres organisateurs simultanément sur ces dossiers.|qBittorrent bietet kein atomares Umbenennen ohne Ersetzen. Sichtbare Ziele werden geprüft; keine anderen Organisierer gleichzeitig auf diese Ordner anwenden.|qBittorrent non offre rinomina atomica senza sostituzione. Si controllano le destinazioni visibili; non usare altri organizzatori contemporaneamente su queste cartelle.|O qBittorrent não oferece mudança de nome atómica sem substituição. São verificados destinos visíveis; não execute outros organizadores simultaneamente nestas pastas.
Choose one source. Download destinations are managed in qBittorrent categories.|Choisissez la source. Les destinations sont définies dans les catégories qBittorrent.|Quelle auswählen. Downloadziele werden in qBittorrent-Kategorien festgelegt.|Scegli la fonte. Le destinazioni sono gestite nelle categorie qBittorrent.|Escolha a origem. Os destinos são geridos nas categorias do qBittorrent.
Receive the library through the public API. No NAS access required.|Recevoir la bibliothèque via l’API publique, sans accès au NAS.|Bibliothek über die öffentliche API empfangen, ohne NAS-Zugriff.|Ricevi la libreria tramite API pubblica, senza accesso al NAS.|Receber a biblioteca pela API pública, sem acesso ao NAS.
Local folders|Dossiers locaux|Lokale Ordner|Cartelle locali|Pastas locais
Read-only scanning. Oberiz never imports, copies or moves media files.|Analyse en lecture seule. Oberiz n’importe, ne copie ni ne déplace les médias.|Nur lesender Scan. Oberiz importiert, kopiert oder verschiebt keine Mediendateien.|Scansione in sola lettura. Oberiz non importa, copia o sposta file multimediali.|Análise só de leitura. O Oberiz não importa, copia nem move ficheiros multimédia.
Enable Public API v1, generate a key and save Settings. In Cinetta, configure the Oberiz URL and key and enable inventory reporting.|Activez l’API publique v1, générez une clé et enregistrez. Configurez l’URL et la clé dans Cinetta et activez l’envoi de l’inventaire.|Öffentliche API v1 aktivieren, Schlüssel erzeugen und speichern. In Cinetta Oberiz-URL und Schlüssel eintragen und Inventarübermittlung aktivieren.|Abilita API pubblica v1, genera una chiave e salva. In Cinetta configura URL e chiave di Oberiz e abilita l’invio dell’inventario.|Ative a API pública v1, gere uma chave e guarde. No Cinetta, configure URL e chave do Oberiz e ative o envio do inventário.
Cinetta sends the inventory; selecting this mode does not initiate a sync.|Cinetta envoie l’inventaire ; sélectionner ce mode ne lance pas la synchronisation.|Cinetta sendet das Inventar; die Moduswahl startet keine Synchronisierung.|Cinetta invia l’inventario; selezionare questa modalità non avvia la sincronizzazione.|O Cinetta envia o inventário; selecionar este modo não inicia a sincronização.
Last inventory received:|Dernier inventaire reçu :|Letztes empfangenes Inventar:|Ultimo inventario ricevuto:|Último inventário recebido:
Not received yet|Pas encore reçu|Noch nicht empfangen|Non ancora ricevuto|Ainda não recebido
If Cinetta stops reporting, the last inventory is preserved. Local scanning is never enabled automatically.|Si Cinetta cesse l’envoi, le dernier inventaire est conservé. L’analyse locale ne s’active jamais automatiquement.|Falls Cinetta nicht mehr sendet, bleibt das letzte Inventar erhalten. Lokale Scans werden nie automatisch aktiviert.|Se Cinetta smette di inviare, l’ultimo inventario viene conservato. La scansione locale non si attiva automaticamente.|Se o Cinetta deixar de enviar, o último inventário é preservado. A análise local nunca é ativada automaticamente.
Optional renaming is configured separately and performed only through qBittorrent. Recovery metadata is included in Oberiz database backups.|Le renommage optionnel est configuré séparément et exécuté par qBittorrent. Les sauvegardes Oberiz incluent les métadonnées de récupération.|Optionales Umbenennen wird separat eingerichtet und nur von qBittorrent ausgeführt. Oberiz-Datenbanksicherungen enthalten Wiederherstellungsmetadaten.|La rinomina opzionale si configura a parte e viene eseguita da qBittorrent. I backup Oberiz includono i metadati di recupero.|A mudança opcional de nomes é configurada à parte e executada pelo qBittorrent. As cópias Oberiz incluem os metadados de recuperação.
No recent Cinetta inventory: automatic acquisition is held until a report is received (maximum age: 24 hours).|Inventaire Cinetta absent ou trop ancien : acquisitions suspendues jusqu’à réception (âge maximal : 24 heures).|Kein aktuelles Cinetta-Inventar: Beschaffung wartet auf einen Bericht (maximal 24 Stunden alt).|Inventario Cinetta assente o vecchio: acquisizioni sospese fino alla ricezione (massimo 24 ore).|Sem inventário recente do Cinetta: aquisições suspensas até à receção (máximo de 24 horas).
qBittorrent naming & recovery|Renommage qBittorrent et récupération|qBittorrent-Benennung und Wiederherstellung|Rinomina qBittorrent e recupero|Nomes pelo qBittorrent e recuperação
Disabled by default. Oberiz requests changes through qBittorrent; it never opens media files.|Désactivé par défaut. Oberiz demande les changements à qBittorrent et n’ouvre jamais les médias.|Standardmäßig deaktiviert. Oberiz beauftragt qBittorrent und öffnet keine Mediendateien.|Disabilitato per impostazione predefinita. Oberiz richiede modifiche a qBittorrent senza aprire i file multimediali.|Desativado por defeito. O Oberiz solicita alterações ao qBittorrent sem abrir ficheiros multimédia.
Rename completed downloads through qBittorrent|Renommer les téléchargements terminés via qBittorrent|Abgeschlossene Downloads über qBittorrent umbenennen|Rinomina download completati tramite qBittorrent|Mudar nomes de transferências concluídas pelo qBittorrent
Apply sharing policies (keep all downloaded files)|Appliquer les règles de partage (conserver tous les fichiers)|Freigaberegeln anwenden (alle Dateien behalten)|Applica regole di condivisione (conserva tutti i file)|Aplicar políticas de partilha (manter todos os ficheiros)
Only Oberiz downloads are processed. Ambiguous files, external subtitles and naming conflicts require manual review.|Seuls les téléchargements Oberiz sont traités. Ambiguïtés, sous-titres externes ou conflits arrêtent le renommage pour vérification.|Nur Oberiz-Downloads werden bearbeitet. Unklare Dateien, externe Untertitel oder Namenskonflikte stoppen die Umbenennung zur Prüfung.|Si elaborano solo i download Oberiz. Ambiguità, sottotitoli esterni o conflitti fermano la rinomina per una verifica.|Só são processadas transferências Oberiz. Ambiguidades, legendas externas ou conflitos interrompem a mudança de nomes para revisão.
Set download destinations in qBittorrent categories. Oberiz does not need NAS credentials.|Définissez les destinations dans les catégories qBittorrent. Oberiz n’a pas besoin des identifiants NAS.|Downloadziele in qBittorrent-Kategorien festlegen. Oberiz benötigt keine NAS-Zugangsdaten.|Configura le destinazioni nelle categorie qBittorrent. Oberiz non richiede credenziali NAS.|Configure os destinos nas categorias do qBittorrent. O Oberiz não precisa de credenciais do NAS.
Downloads & recovery|Téléchargements et récupération|Downloads und Wiederherstellung|Download e recupero|Transferências e recuperação
Torrent metadata, destinations and naming history. Recovery never starts downloading automatically.|Métadonnées torrent, destinations et historique des noms. La récupération ne lance jamais un téléchargement automatiquement.|Torrent-Metadaten, Ziele und Namensverlauf. Die Wiederherstellung startet nie automatisch Downloads.|Metadati torrent, destinazioni e cronologia dei nomi. Il recupero non avvia download automaticamente.|Metadados torrent, destinos e histórico de nomes. A recuperação nunca inicia transferências automaticamente.
Restore stopped|Restaurer à l’arrêt|Angehalten wiederherstellen|Ripristina arrestato|Recuperar parado
Restore this torrent stopped in qBittorrent, restore its recorded names and request a recheck? Inspect the result before resuming manually.|Restaurer ce torrent à l’arrêt dans qBittorrent, ses noms et demander une vérification ? Vérifiez le résultat avant de reprendre manuellement.|Diesen Torrent angehalten in qBittorrent mit gespeicherten Namen wiederherstellen und prüfen lassen? Vor manuellem Fortsetzen Ergebnis kontrollieren.|Ripristinare questo torrent arrestato in qBittorrent con i nomi salvati e richiedere una verifica? Controlla prima di riprendere manualmente.|Recuperar este torrent parado no qBittorrent com os nomes guardados e pedir uma verificação? Reveja antes de retomar manualmente.
Restored stopped in {0}. Recheck requested; inspect and resume in qBittorrent.|Restauré à l’arrêt dans {0}. Vérification demandée ; contrôlez puis reprenez dans qBittorrent.|Angehalten in {0} wiederhergestellt. Prüfung angefordert; in qBittorrent kontrollieren und fortsetzen.|Ripristinato arrestato in {0}. Verifica richiesta; controlla e riprendi in qBittorrent.|Recuperado parado em {0}. Verificação pedida; reveja e retome no qBittorrent.
Downloaded files are always kept.|Les fichiers téléchargés sont toujours conservés.|Heruntergeladene Dateien bleiben immer erhalten.|I file scaricati vengono sempre conservati.|Os ficheiros transferidos são sempre mantidos.
Dashboard|Tableau de bord|Übersicht|Panoramica|Painel
SMB / CIFS servers|Serveurs SMB / CIFS|SMB-/CIFS-Server|Server SMB / CIFS|Servidores SMB / CIFS
Server|Serveur|Server|Server|Servidor
Open|Ouvrir|Öffnen|Apri|Abrir
Forget saved connection|Oublier la connexion|Gespeicherte Verbindung entfernen|Dimentica connessione salvata|Esquecer ligação guardada
No mapping needed: Oberiz already opens every folder qBittorrent uses, or no matching shared folder was found.|Aucune correspondance nécessaire : Oberiz ouvre déjà tous les dossiers utilisés par qBittorrent, ou aucun dossier partagé correspondant n'a été trouvé.|Keine Zuordnung nötig: Oberiz öffnet bereits alle Ordner, die qBittorrent verwendet, oder es wurde kein passender freigegebener Ordner gefunden.|Nessuna mappatura necessaria: Oberiz apre già tutte le cartelle usate da qBittorrent, oppure non è stata trovata alcuna cartella condivisa corrispondente.|Nenhum mapeamento necessário: o Oberiz já abre todas as pastas usadas pelo qBittorrent, ou não foi encontrada nenhuma pasta partilhada correspondente.
{0} mappings added. Save Settings to apply them.|{0} correspondances ajoutées. Enregistrez les paramètres pour les appliquer.|{0} Zuordnungen hinzugefügt. Speichern Sie die Einstellungen, um sie anzuwenden.|{0} mappature aggiunte. Salva le impostazioni per applicarle.|{0} mapeamentos adicionados. Guarde as definições para os aplicar.
Files and storage|Fichiers et stockage|Dateien und Speicher|File e archiviazione|Ficheiros e armazenamento
Where qBittorrent downloads, where your movies and series end up, and how Oberiz reaches them.|Où qBittorrent télécharge, où arrivent vos films et séries, et comment Oberiz y accède.|Wo qBittorrent herunterlädt, wo Ihre Filme und Serien landen und wie Oberiz sie erreicht.|Dove scarica qBittorrent, dove finiscono i tuoi film e serie e come Oberiz li raggiunge.|Onde o qBittorrent transfere, onde ficam os seus filmes e séries e como o Oberiz chega até eles.
Should Oberiz move and rename downloads?|Oberiz doit-il déplacer et renommer les téléchargements ?|Soll Oberiz Downloads verschieben und umbenennen?|Oberiz deve spostare e rinominare i download?|O Oberiz deve mover e renomear as transferências?
Yes|Oui|Ja|Sì|Sim
Oberiz imports each completed download into your library folder and renames it.|Oberiz importe chaque téléchargement terminé dans le dossier de votre bibliothèque et le renomme.|Oberiz importiert jeden abgeschlossenen Download in Ihren Bibliotheksordner und benennt ihn um.|Oberiz importa ogni download completato nella cartella della libreria e lo rinomina.|O Oberiz importa cada transferência concluída para a pasta da biblioteca e renomeia-a.
No|Non|Nein|No|Não
qBittorrent downloads straight into your library and your media server (Cinetta) tells Oberiz what you have.|qBittorrent télécharge directement dans votre bibliothèque et votre serveur multimédia (Cinetta) indique à Oberiz ce que vous avez.|qBittorrent lädt direkt in Ihre Bibliothek herunter und Ihr Medienserver (Cinetta) teilt Oberiz mit, was vorhanden ist.|qBittorrent scarica direttamente nella libreria e il tuo media server (Cinetta) comunica a Oberiz cosa hai.|O qBittorrent transfere diretamente para a biblioteca e o seu servidor multimédia (Cinetta) diz ao Oberiz o que tem.
Oberiz will never open, move, rename or delete files. Point each qBittorrent category directly at a library folder. Folders, NAS access and path mappings are not needed for downloads.|Oberiz n'ouvrira, ne déplacera, ne renommera ni ne supprimera jamais de fichiers. Faites pointer chaque catégorie qBittorrent directement vers un dossier de la bibliothèque. Les dossiers, l'accès au NAS et les correspondances de chemins ne sont pas nécessaires pour les téléchargements.|Oberiz öffnet, verschiebt, benennt oder löscht niemals Dateien. Richten Sie jede qBittorrent-Kategorie direkt auf einen Bibliotheksordner. Ordner, NAS-Zugang und Pfadzuordnungen werden für Downloads nicht benötigt.|Oberiz non aprirà, sposterà, rinominerà né eliminerà mai file. Imposta ogni categoria di qBittorrent direttamente su una cartella della libreria. Cartelle, accesso al NAS e mappature dei percorsi non servono per i download.|O Oberiz nunca abrirá, moverá, renomeará nem eliminará ficheiros. Aponte cada categoria do qBittorrent diretamente para uma pasta da biblioteca. Pastas, acesso ao NAS e mapeamentos de caminhos não são necessários para as transferências.
Where does qBittorrent run?|Où s'exécute qBittorrent ?|Wo läuft qBittorrent?|Dove si trova qBittorrent?|Onde está o qBittorrent?
On this computer|Sur cet ordinateur|Auf diesem Computer|Su questo computer|Neste computador
Oberiz and qBittorrent see the same paths. Nothing to configure.|Oberiz et qBittorrent voient les mêmes chemins. Rien à configurer.|Oberiz und qBittorrent sehen dieselben Pfade. Nichts zu konfigurieren.|Oberiz e qBittorrent vedono gli stessi percorsi. Niente da configurare.|O Oberiz e o qBittorrent veem os mesmos caminhos. Nada a configurar.
On another computer or the NAS|Sur un autre ordinateur ou le NAS|Auf einem anderen Computer oder dem NAS|Su un altro computer o sul NAS|Noutro computador ou no NAS
qBittorrent reports its own paths, which Oberiz must translate into folders it can open.|qBittorrent indique ses propres chemins, qu'Oberiz doit traduire en dossiers qu'il peut ouvrir.|qBittorrent meldet seine eigenen Pfade, die Oberiz in Ordner übersetzen muss, die es öffnen kann.|qBittorrent indica i propri percorsi, che Oberiz deve tradurre in cartelle che può aprire.|O qBittorrent indica os seus próprios caminhos, que o Oberiz tem de traduzir para pastas que consiga abrir.
Example: qBittorrent sees /volume1/WD19TB and Oberiz sees|Exemple : qBittorrent voit /volume1/WD19TB et Oberiz voit|Beispiel: qBittorrent sieht /volume1/WD19TB und Oberiz sieht|Esempio: qBittorrent vede /volume1/WD19TB e Oberiz vede|Exemplo: o qBittorrent vê /volume1/WD19TB e o Oberiz vê
Detecting…|Détection…|Wird erkannt…|Rilevamento…|A detetar…
Detect automatically|Détecter automatiquement|Automatisch erkennen|Rileva automaticamente|Detetar automaticamente
Suggested mappings|Correspondances suggérées|Vorgeschlagene Zuordnungen|Mappature suggerite|Mapeamentos sugeridos
Checked: {0} opens correctly.|Vérifié : {0} s'ouvre correctement.|Geprüft: {0} lässt sich öffnen.|Verificata: {0} si apre correttamente.|Verificado: {0} abre corretamente.
Not verified: {0} could not be opened. If the NAS needs a password, connect it in the NAS access step first.|Non vérifié : impossible d'ouvrir {0}. Si le NAS demande un mot de passe, connectez-le d'abord à l'étape d'accès au NAS.|Nicht geprüft: {0} konnte nicht geöffnet werden. Wenn das NAS ein Passwort verlangt, verbinden Sie es zuerst im Schritt NAS-Zugang.|Non verificata: impossibile aprire {0}. Se il NAS richiede una password, collegalo prima nel passaggio di accesso al NAS.|Não verificado: não foi possível abrir {0}. Se o NAS pedir palavra-passe, ligue-o primeiro no passo de acesso ao NAS.
Add selected mappings|Ajouter les correspondances sélectionnées|Ausgewählte Zuordnungen hinzufügen|Aggiungi le mappature selezionate|Adicionar os mapeamentos selecionados
Folders|Dossiers|Ordner|Cartelle|Pastas
Where qBittorrent leaves completed downloads, as Oberiz sees it.|Où qBittorrent laisse les téléchargements terminés, tel qu'Oberiz le voit.|Wo qBittorrent abgeschlossene Downloads ablegt, aus Sicht von Oberiz.|Dove qBittorrent lascia i download completati, come lo vede Oberiz.|Onde o qBittorrent deixa as transferências concluídas, tal como o Oberiz o vê.
Where movies end up, already renamed.|Où arrivent les films, déjà renommés.|Wo Filme landen, bereits umbenannt.|Dove finiscono i film, già rinominati.|Onde ficam os filmes, já renomeados.
Where series end up, already renamed.|Où arrivent les séries, déjà renommées.|Wo Serien landen, bereits umbenannt.|Dove finiscono le serie, già rinominate.|Onde ficam as séries, já renomeadas.
NAS access (SMB)|Accès au NAS (SMB)|NAS-Zugang (SMB)|Accesso al NAS (SMB)|Acesso ao NAS (SMB)
The NAS username and password, so the Oberiz service can open folders that start with \\. Not needed for folders on this computer.|L'identifiant et le mot de passe du NAS, pour que le service Oberiz puisse ouvrir les dossiers qui commencent par \\. Inutile pour les dossiers de cet ordinateur.|Benutzername und Passwort des NAS, damit der Oberiz-Dienst Ordner öffnen kann, die mit \\ beginnen. Für Ordner auf diesem Computer nicht nötig.|Nome utente e password del NAS, così il servizio Oberiz può aprire le cartelle che iniziano con \\. Non serve per le cartelle di questo computer.|O utilizador e a palavra-passe do NAS, para que o serviço Oberiz consiga abrir as pastas que começam por \\. Não é necessário para pastas deste computador.
Configure NAS access (SMB)|Configurer l'accès au NAS (SMB)|NAS-Zugang (SMB) einrichten|Configura accesso al NAS (SMB)|Configurar acesso ao NAS (SMB)
Advanced options|Options avancées|Erweiterte Optionen|Opzioni avanzate|Opções avançadas
Existing library (optional)|Bibliothèque existante (facultatif)|Vorhandene Bibliothek (optional)|Libreria esistente (facoltativo)|Biblioteca existente (opcional)
Folders Oberiz scans to detect what you already had. Not needed if your media server (Cinetta) reports your library.|Dossiers qu'Oberiz analyse pour détecter ce que vous aviez déjà. Inutile si votre serveur multimédia (Cinetta) déclare votre bibliothèque.|Ordner, die Oberiz durchsucht, um bereits Vorhandenes zu erkennen. Nicht nötig, wenn Ihr Medienserver (Cinetta) Ihre Bibliothek meldet.|Cartelle che Oberiz analizza per rilevare ciò che avevi già. Non serve se il tuo media server (Cinetta) comunica la tua libreria.|Pastas que o Oberiz analisa para detetar o que já tinha. Não é necessário se o seu servidor multimédia (Cinetta) comunicar a sua biblioteca.
new path mapping|nouvelle correspondance de chemin|neue Pfadzuordnung|nuova mappatura del percorso|novo mapeamento de caminho
Create the path mappings these folders need automatically|Créer automatiquement les correspondances de chemins nécessaires à ces dossiers|Benötigte Pfadzuordnungen für diese Ordner automatisch anlegen|Crea automaticamente le mappature dei percorsi necessarie a queste cartelle|Criar automaticamente os mapeamentos de caminhos de que estas pastas precisam
If qBittorrent runs on another computer or the NAS, set its path mappings in Files and storage.|Si qBittorrent s'exécute sur un autre ordinateur ou le NAS, définissez ses correspondances de chemins dans Fichiers et stockage.|Wenn qBittorrent auf einem anderen Computer oder dem NAS läuft, legen Sie die Pfadzuordnungen unter Dateien und Speicher fest.|Se qBittorrent si trova su un altro computer o sul NAS, imposta le mappature dei percorsi in File e archiviazione.|Se o qBittorrent estiver noutro computador ou no NAS, defina os mapeamentos de caminhos em Ficheiros e armazenamento.
Oberiz does not manage files: completed downloads stay where qBittorrent saved them, and seed cleanup never deletes their data. Change it in Files and storage.|Oberiz ne gère pas les fichiers : les téléchargements terminés restent là où qBittorrent les a enregistrés et le nettoyage après partage ne supprime jamais leurs données. Modifiable dans Fichiers et stockage.|Oberiz verwaltet keine Dateien: Abgeschlossene Downloads bleiben dort, wo qBittorrent sie gespeichert hat, und die Bereinigung nach dem Seeden löscht nie ihre Daten. Änderbar unter Dateien und Speicher.|Oberiz non gestisce i file: i download completati restano dove qBittorrent li ha salvati e la pulizia dopo il seeding non elimina mai i loro dati. Si cambia in File e archiviazione.|O Oberiz não gere ficheiros: as transferências concluídas ficam onde o qBittorrent as guardou e a limpeza após a partilha nunca elimina os seus dados. Altere-o em Ficheiros e armazenamento.
Indexer folders|Dossiers des indexeurs|Indexer-Ordner|Cartelle degli indexer|Pastas dos indexadores
Where tracker definitions are read from. Normally there is no need to change them.|Emplacement des définitions de trackers. Normalement, inutile de les modifier.|Von hier werden Tracker-Definitionen gelesen. Normalerweise müssen sie nicht geändert werden.|Da dove vengono lette le definizioni dei tracker. Di norma non serve cambiarle.|De onde são lidas as definições dos trackers. Normalmente não é preciso alterá-las.
Your private tracker definitions (.yml). They are never published or synchronized.|Vos définitions de trackers privés (.yml). Elles ne sont jamais publiées ni synchronisées.|Ihre privaten Tracker-Definitionen (.yml). Sie werden nie veröffentlicht oder synchronisiert.|Le tue definizioni private dei tracker (.yml). Non vengono mai pubblicate né sincronizzate.|As suas definições privadas de trackers (.yml). Nunca são publicadas nem sincronizadas.
Public definitions Oberiz downloads when you synchronize the catalogue.|Définitions publiques téléchargées par Oberiz lors de la synchronisation du catalogue.|Öffentliche Definitionen, die Oberiz beim Synchronisieren des Katalogs herunterlädt.|Definizioni pubbliche che Oberiz scarica quando sincronizzi il catalogo.|Definições públicas que o Oberiz transfere ao sincronizar o catálogo.
Forget this saved SMB connection? Existing Windows sessions will not be disconnected.|Oublier cette connexion SMB ? Les sessions Windows existantes ne seront pas déconnectées.|Diese SMB-Verbindung entfernen? Bestehende Windows-Sitzungen werden nicht getrennt.|Dimenticare questa connessione SMB? Le sessioni Windows esistenti non verranno disconnesse.|Esquecer esta ligação SMB? As sessões Windows existentes não serão desligadas.
SMB connection saved|Connexion SMB enregistrée|SMB-Verbindung gespeichert|Connessione SMB salvata|Ligação SMB guardada
Use HTTPS when entering credentials remotely. Saved passwords are protected by Windows; shared folders appear in the folder picker.|Utilisez HTTPS pour saisir les identifiants à distance. Windows protège les mots de passe ; les partages apparaissent dans le sélecteur.|Nutzen Sie HTTPS für entfernte Anmeldedaten. Windows schützt gespeicherte Passwörter; Freigaben erscheinen im Ordnerdialog.|Usa HTTPS per inserire credenziali da remoto. Windows protegge le password; le cartelle condivise appaiono nel selettore.|Use HTTPS ao introduzir credenciais remotamente. O Windows protege as palavras-passe; as pastas partilhadas aparecem no seletor.
Share name (optional)|Nom du partage (facultatif)|Freigabename (optional)|Nome condivisione (facoltativo)|Nome da partilha (opcional)
Leave empty to keep the saved password|Laissez vide pour conserver le mot de passe|Leer lassen, um das Passwort beizubehalten|Lascia vuoto per mantenere la password|Deixe vazio para manter a palavra-passe
Connecting…|Connexion…|Verbindung wird hergestellt…|Connessione…|A ligar…
Connect and save|Connecter et enregistrer|Verbinden und speichern|Connetti e salva|Ligar e guardar
No shares discovered|Aucun partage trouvé|Keine Freigaben gefunden|Nessuna condivisione trovata|Nenhuma partilha encontrada
All storage locations|Tous les disques et partages|Alle Laufwerke und Freigaben|Tutti i dischi e le condivisioni|Todos os discos e partilhas
Folder path|Chemin du dossier|Ordnerpfad|Percorso cartella|Caminho da pasta
Reset|Réinitialiser|Zurücksetzen|Reimposta|Repor
Import YAML|Importer YAML|YAML importieren|Importa YAML|Importar YAML
Choose a YAML definition up to 512 KiB.|Choisissez une définition YAML de 512 Kio maximum.|Wählen Sie eine YAML-Definition mit höchstens 512 KiB.|Scegli una definizione YAML fino a 512 KiB.|Selecione uma definição YAML até 512 KiB.
Replace the existing custom definition? Saved credentials will be preserved.|Remplacer la définition personnalisée existante ? Les identifiants enregistrés seront conservés.|Vorhandene benutzerdefinierte Definition ersetzen? Gespeicherte Zugangsdaten bleiben erhalten.|Sostituire la definizione personalizzata esistente? Le credenziali salvate saranno conservate.|Substituir a definição personalizada existente? As credenciais guardadas serão preservadas.
Episodes|Épisodes|Episoden|Episodi|Episódios
{0} missing · {1} future|{0} manquants · {1} à venir|{0} fehlen · {1} künftig|{0} mancanti · {1} futuri|{0} em falta · {1} futuros
Episodes|Épisodes|Episoden|Episodi|Episódios
{0} missing · {1} future|{0} manquants · {1} à venir|{0} fehlen · {1} künftig|{0} mancanti · {1} futuri|{0} em falta · {1} futuros
Add folders|Ajouter des dossiers|Ordner hinzufügen|Aggiungi cartelle|Adicionar pastas
Select current folder|Sélectionner ce dossier|Diesen Ordner auswählen|Seleziona questa cartella|Selecionar esta pasta
{0} folders selected|{0} dossiers sélectionnés|{0} Ordner ausgewählt|{0} cartelle selezionate|{0} pastas selecionadas
Add selected folders|Ajouter les dossiers sélectionnés|Ausgewählte Ordner hinzufügen|Aggiungi cartelle selezionate|Adicionar pastas selecionadas
Maximum 64 folders per media type.|Maximum 64 dossiers par type de média.|Maximal 64 Ordner pro Medientyp.|Massimo 64 cartelle per tipo di contenuto.|Máximo de 64 pastas por tipo de conteúdo.
Complete both paths in each mapping, or remove the row.|Complétez les deux chemins de chaque correspondance ou supprimez la ligne.|Beide Pfade jeder Zuordnung ausfüllen oder die Zeile entfernen.|Completa entrambi i percorsi di ogni mappatura o rimuovi la riga.|Preencha ambos os caminhos de cada mapeamento ou remova a linha.
Movie import destination|Destination d'importation des films|Importziel für Filme|Destinazione importazione film|Destino de importação de filmes
Series import destination|Destination d'importation des séries|Importziel für Serien|Destinazione importazione serie|Destino de importação de séries
Library folders|Dossiers de la bibliothèque|Bibliotheksordner|Cartelle della libreria|Pastas da biblioteca
Library folder|Dossier de la bibliothèque|Bibliotheksordner|Cartella della libreria|Pasta da biblioteca
Add every disk containing existing media. Scanning never moves files.|Ajoutez tous les disques contenant des médias. L'analyse ne déplace jamais les fichiers.|Alle Datenträger mit vorhandenen Medien hinzufügen. Scans verschieben niemals Dateien.|Aggiungi tutti i dischi con contenuti esistenti. La scansione non sposta mai i file.|Adicione todos os discos com conteúdos existentes. A análise nunca move ficheiros.
Remote path mappings|Correspondances des chemins distants|Zuordnungen entfernter Pfade|Mappature dei percorsi remoti|Mapeamentos de caminhos remotos
Map qBittorrent paths to folders accessible by the Oberiz server. Categories and torrent states are not changed.|Associez les chemins qBittorrent aux dossiers accessibles au serveur Oberiz. Les catégories et états des torrents ne changent pas.|qBittorrent-Pfade zugänglichen Ordnern des Oberiz-Servers zuordnen. Kategorien und Torrent-Zustände bleiben unverändert.|Associa i percorsi qBittorrent alle cartelle accessibili dal server Oberiz. Categorie e stati dei torrent non cambiano.|Associe os caminhos qBittorrent às pastas acessíveis pelo servidor Oberiz. As categorias e os estados dos torrents não são alterados.
Add mapping|Ajouter une correspondance|Zuordnung hinzufügen|Aggiungi mappatura|Adicionar mapeamento
Add folder|Ajouter un dossier|Ordner hinzufügen|Aggiungi cartella|Adicionar pasta
Load category folders|Charger les dossiers des catégories|Kategorieordner laden|Carica cartelle categorie|Carregar pastas das categorias
Add from qBittorrent category|Ajouter depuis une catégorie qBittorrent|Aus qBittorrent-Kategorie hinzufügen|Aggiungi da categoria qBittorrent|Adicionar da categoria qBittorrent
Schedule library scans|Planifier les analyses|Bibliotheksscans planen|Pianifica scansioni libreria|Agendar análises da biblioteca
Unavailable disks preserve indexed files and hold automatic acquisition to avoid duplicate downloads.|Les disques indisponibles conservent leurs fichiers indexés et suspendent l'acquisition automatique pour éviter les doublons.|Nicht verfügbare Datenträger behalten indizierte Dateien; automatische Downloads pausieren zur Vermeidung von Duplikaten.|I dischi non disponibili mantengono i file indicizzati e sospendono l'acquisizione automatica per evitare duplicati.|Os discos indisponíveis mantêm os ficheiros indexados e suspendem a aquisição automática para evitar duplicados.
Accessible|Accessible|Zugänglich|Accessibile|Acessível
Not scanned yet|Pas encore analysé|Noch nicht gescannt|Non ancora scansionata|Ainda não analisada
Monitored Series|Séries suivies|Überwachte Serien|Serie monitorate|Séries monitorizadas
Available episodes|Épisodes disponibles|Verfügbare Folgen|Episodi disponibili|Episódios disponíveis
{0} aired missing · {1} future|{0} diffusés manquants · {1} à venir|{0} ausgestrahlte fehlen · {1} zukünftige|{0} trasmessi mancanti · {1} futuri|{0} emitidos em falta · {1} futuros
Size of indexed media files, not disk capacity or all torrents.|Taille des fichiers médias indexés, pas la capacité des disques ni tous les torrents.|Größe indizierter Mediendateien, nicht Datenträgerkapazität oder alle Torrents.|Dimensione dei file multimediali indicizzati, non capacità dei dischi o tutti i torrent.|Tamanho dos ficheiros multimédia indexados, não a capacidade dos discos ou todos os torrents.
Configure folders and scan|Configurer les dossiers et analyser|Ordner konfigurieren und scannen|Configura cartelle e scansiona|Configurar pastas e analisar
Season complete|Saison complète|Staffel vollständig|Stagione completa|Temporada completa
Up to date|À jour|Auf aktuellem Stand|Aggiornata|Em dia
Partially available|Partiellement disponible|Teilweise verfügbar|Parzialmente disponibile|Parcialmente disponível
Follow upcoming episodes|Suivre les prochains épisodes|Kommende Folgen verfolgen|Segui prossimi episodi|Seguir próximos episódios
The server is starting or reconnecting. This page will continue automatically.|Le serveur démarre ou se reconnecte. Cette page continuera automatiquement.|Der Server startet oder verbindet sich erneut. Diese Seite wird automatisch fortgesetzt.|Il server si sta avviando o riconnettendo. La pagina continuerà automaticamente.|O servidor está a iniciar ou a restabelecer a ligação. Esta página continuará automaticamente.
Remove configured indexer {0}? The definition remains in Add Indexer. Seed policies and download history are preserved.|Supprimer l'indexeur configuré {0} ? Sa définition reste dans Ajouter un indexeur. Les règles de partage et l'historique sont conservés.|Konfigurierten Indexer {0} entfernen? Die Definition bleibt unter Indexer hinzufügen verfügbar. Seeding-Richtlinien und Download-Verlauf bleiben erhalten.|Rimuovere l'indicizzatore configurato {0}? La definizione resta in Aggiungi indicizzatore. Regole di condivisione e cronologia restano intatte.|Remover o indexador configurado {0}? A definição mantém-se em Adicionar indexador. As políticas de partilha e o histórico são preservados.
Too many failed sign-in attempts. Try again later.|Trop de tentatives de connexion échouées. Réessayez plus tard.|Zu viele fehlgeschlagene Anmeldeversuche. Später erneut versuchen.|Troppi tentativi di accesso falliti. Riprova più tardi.|Demasiadas tentativas de acesso falhadas. Tente novamente mais tarde.
. Endpoints: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.|. Routes : GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.|. Endpunkte: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.|. Endpoint: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.|. Rotas: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.
Movies (Radarr): http://your-oberiz-host:2032/radarr|Films (Radarr) : http://your-oberiz-host:2032/radarr|Filme (Radarr): http://your-oberiz-host:2032/radarr|Film (Radarr): http://your-oberiz-host:2032/radarr|Filmes (Radarr): http://your-oberiz-host:2032/radarr
Series (Sonarr): http://your-oberiz-host:2032/sonarr|Séries (Sonarr) : http://your-oberiz-host:2032/sonarr|Serien (Sonarr): http://your-oberiz-host:2032/sonarr|Serie (Sonarr): http://your-oberiz-host:2032/sonarr|Séries (Sonarr): http://your-oberiz-host:2032/sonarr
Z:\WD19TB or \\nas\WD19TB|Z:\WD19TB ou \\nas\WD19TB|Z:\WD19TB oder \\nas\WD19TB|Z:\WD19TB o \\nas\WD19TB|Z:\WD19TB ou \\nas\WD19TB
e.g. my-private-tracker|p. ex. mon-tracker-prive|z. B. mein-privater-tracker|ad es. mio-tracker-privato|p. ex. meu-tracker-privado
Only use RSS for this indexer (skip it in the scheduled automatic search)|Utiliser uniquement RSS pour cet indexeur (ignorer la recherche automatique planifiée)|Für diesen Indexer nur RSS verwenden (geplante automatische Suche überspringen)|Usa solo RSS per questo indicizzatore (salta la ricerca automatica pianificata)|Usar apenas RSS para este indexador (ignorar a pesquisa automática agendada)
 · cutoff {0}| · seuil {0}| · Grenzwert {0}| · limite {0}| · limite {0}
Completed · {0} missing · {1} searched · {2} grabbed · {3} skipped · {4} errors · {5} monitored|Terminé · {0} manquants · {1} recherchés · {2} envoyés · {3} ignorés · {4} erreurs · {5} suivis|Abgeschlossen · {0} fehlend · {1} gesucht · {2} übergeben · {3} übersprungen · {4} Fehler · {5} überwacht|Completato · {0} mancanti · {1} cercati · {2} inviati · {3} saltati · {4} errori · {5} monitorati|Concluído · {0} em falta · {1} pesquisados · {2} enviados · {3} ignorados · {4} erros · {5} monitorizados
Connected · qBittorrent {0} · {1} ms · {2}|Connecté · qBittorrent {0} · {1} ms · {2}|Verbunden · qBittorrent {0} · {1} ms · {2}|Connesso · qBittorrent {0} · {1} ms · {2}|Ligado · qBittorrent {0} · {1} ms · {2}
Delete local backup {0}? This cannot be undone.|Supprimer la sauvegarde locale {0} ? Cette action est irréversible.|Lokale Sicherung {0} löschen? Dies kann nicht rückgängig gemacht werden.|Eliminare il backup locale {0}? L'operazione è irreversibile.|Eliminar a cópia local {0}? Não é possível desfazer.
RSS run · {0} feeds · {1} new · {2} matched · {3} grabbed · {4} errors · {5} configured|Exécution RSS · {0} flux · {1} nouveaux · {2} associés · {3} envoyés · {4} erreurs · {5} configurés|RSS-Lauf · {0} Feeds · {1} neu · {2} zugeordnet · {3} übergeben · {4} Fehler · {5} konfiguriert|Esecuzione RSS · {0} feed · {1} nuovi · {2} associati · {3} inviati · {4} errori · {5} configurati|Execução RSS · {0} feeds · {1} novos · {2} associados · {3} enviados · {4} erros · {5} configurados
Remove the category “{0}”? Torrents will remain in qBittorrent but become uncategorized.|Supprimer la catégorie « {0} » ? Les torrents resteront dans qBittorrent sans catégorie.|Kategorie „{0}“ entfernen? Die Torrents bleiben in qBittorrent ohne Kategorie.|Rimuovere la categoria «{0}»? I torrent resteranno in qBittorrent senza categoria.|Remover a categoria «{0}»? Os torrents permanecerão no qBittorrent sem categoria.
Remove the tag “{0}” from qBittorrent?|Supprimer l'étiquette « {0} » de qBittorrent ?|Tag „{0}“ aus qBittorrent entfernen?|Rimuovere l'etichetta «{0}» da qBittorrent?|Remover a etiqueta «{0}» do qBittorrent?
Remove {0} obsolete Oberiz record{1}? qBittorrent torrents and media files will not be changed.|Supprimer {0} enregistrements obsolètes d'Oberiz ? Les torrents et fichiers multimédias ne seront pas modifiés.|{0} veraltete Oberiz-Einträge entfernen? Torrents und Mediendateien werden nicht verändert.|Rimuovere {0} record obsoleti di Oberiz? I torrent e i file multimediali non saranno modificati.|Remover {0} registos obsoletos do Oberiz? Os torrents e ficheiros multimédia não serão alterados.
Restore {0}? This replaces the current Oberiz database with that backup. Current media, requests, settings and history will be reverted.|Restaurer {0} ? La base de données actuelle sera remplacée par cette sauvegarde. Contenus, demandes, paramètres et historique seront restaurés.|{0} wiederherstellen? Die aktuelle Datenbank wird durch diese Sicherung ersetzt. Medien, Anfragen, Einstellungen und Verlauf werden zurückgesetzt.|Ripristinare {0}? Il database attuale sarà sostituito dal backup. Contenuti, richieste, impostazioni e cronologia saranno ripristinati.|Restaurar {0}? A base de dados atual será substituída pela cópia. Conteúdo, pedidos, definições e histórico serão repostos.
{0} enabled (not a live health check)|{0} activés (sans vérification en temps réel)|{0} aktiviert (keine Echtzeitprüfung)|{0} attivi (non è una verifica in tempo reale)|{0} ativos (não é uma verificação em tempo real)
Stored permanently in Oberiz settings.|Enregistré dans les paramètres d'Oberiz.|Dauerhaft in den Oberiz-Einstellungen gespeichert.|Salvato nelle impostazioni di Oberiz.|Guardado nas definições do Oberiz.
Services, automation, importing, seeding and appearance.|Services, automatisation, importation, partage et apparence.|Dienste, Automatisierung, Import, Seeding und Darstellung.|Servizi, automazione, importazione, condivisione e aspetto.|Serviços, automatização, importação, partilha e aparência.
. Library metadata stays usable; download progress is paused.|. Les métadonnées restent accessibles ; le suivi des téléchargements est suspendu.|. Bibliotheksmetadaten bleiben verfügbar; der Download-Fortschritt ist pausiert.|. I metadati restano disponibili; il progresso dei download è sospeso.|. Os metadados permanecem disponíveis; o progresso das transferências está suspenso.
. Restoring replaces all current database data; media files and qBittorrent are not changed.|. La restauration remplace la base de données ; les fichiers et qBittorrent ne sont pas modifiés.|. Die Wiederherstellung ersetzt alle Datenbankdaten; Mediendateien und qBittorrent bleiben unverändert.|. Il ripristino sostituisce tutti i dati del database; file e qBittorrent restano invariati.|. O restauro substitui os dados da base de dados; os ficheiros e qBittorrent não são alterados.
. Season packs are imported file-by-file when episode numbers can be detected.|. Les packs de saisons sont importés fichier par fichier si les numéros d'épisode sont détectés.|. Staffelpakete werden Datei für Datei importiert, wenn Episodennummern erkannt werden.|. I pacchetti delle stagioni sono importati file per file se si rilevano i numeri degli episodi.|. Os pacotes de temporada são importados ficheiro a ficheiro quando se detetam os números dos episódios.
. Series also supports|. Les séries prennent aussi en charge|. Serien unterstützen außerdem|. Le serie supportano anche|. As séries também suportam
Add a movie or series to start tracking wanted media here.|Ajoutez un film ou une série pour commencer le suivi du contenu recherché.|Film oder Serie hinzufügen, um gesuchte Medien hier zu überwachen.|Aggiungi un film o una serie per monitorare i contenuti richiesti.|Adicione um filme ou série para monitorizar o conteúdo procurado.
Allow Oberiz to pick this release again|Permettre à Oberiz de choisir cette version à nouveau|Oberiz darf diese Veröffentlichung wieder wählen|Consenti a Oberiz di scegliere nuovamente questa versione|Permitir ao Oberiz escolher esta versão novamente
Allow releases where language cannot be identified|Autoriser les versions dont la langue est inconnue|Veröffentlichungen mit unbekannter Sprache erlauben|Consenti versioni con lingua non identificabile|Permitir versões cujo idioma não pode ser identificado
Allowed languages and language-specific scoring.|Langues autorisées et scores par langue.|Erlaubte Sprachen und sprachabhängige Punktzahlen.|Lingue consentite e punteggi per lingua.|Idiomas permitidos e pontuações por idioma.
Automatically search monitored media using its assigned profile.|Rechercher automatiquement les contenus suivis avec leur profil.|Überwachte Medien automatisch mit ihrem Profil suchen.|Cerca automaticamente i contenuti monitorati usando il loro profilo.|Pesquisar automaticamente conteúdo monitorizado com o perfil atribuído.
Category controls destination. Tags can use variables.|La catégorie détermine la destination. Les étiquettes acceptent des variables.|Die Kategorie bestimmt das Ziel. Tags können Variablen verwenden.|La categoria determina la destinazione. Le etichette supportano variabili.|A categoria determina o destino. As etiquetas aceitam variáveis.
Cleanup downloads after seed policy is satisfied|Nettoyer les téléchargements une fois les règles de partage respectées|Downloads nach Erfüllung der Seeding-Richtlinie bereinigen|Pulisci download quando la regola di condivisione è soddisfatta|Limpar transferências após cumprir a política de partilha
Comma separated. Variables are resolved at Grab time.|Séparées par des virgules. Les variables sont résolues au téléchargement.|Durch Kommas getrennt. Variablen werden beim Download aufgelöst.|Separate da virgole. Le variabili sono risolte al download.|Separadas por vírgulas. As variáveis são resolvidas ao transferir.
Completed downloads are imported without breaking active seeding.|Les téléchargements terminés sont importés sans interrompre le partage.|Abgeschlossene Downloads werden ohne Unterbrechung des Seedings importiert.|I download completati sono importati senza interrompere la condivisione.|As transferências concluídas são importadas sem interromper a partilha.
Completed imports, seed cleanup and recoverable torrent metadata.|Importations terminées, nettoyage du partage et métadonnées réutilisables.|Abgeschlossene Importe, Seeding-Bereinigung und wiederverwendbare Torrent-Metadaten.|Importazioni completate, pulizia condivisione e metadati torrent riutilizzabili.|Importações concluídas, limpeza de partilha e metadados torrent reutilizáveis.
Configured indexers first. Definitions stay in the Add Indexer catalogue.|Les indexeurs configurés en premier. Les définitions restent dans Ajouter un indexeur.|Konfigurierte Indexer zuerst. Definitionen bleiben im Katalog Indexer hinzufügen.|Prima gli indicizzatori configurati. Le definizioni restano nel catalogo Aggiungi indicizzatore.|Primeiro os indexadores configurados. As definições mantêm-se no catálogo Adicionar indexador.
Consistent SQLite snapshots, including data currently in WAL.|Sauvegardes SQLite cohérentes, y compris les données du WAL.|Konsistente SQLite-Sicherungen einschließlich der WAL-Daten.|Backup SQLite coerenti, inclusi i dati nel WAL.|Cópias SQLite consistentes, incluindo os dados no WAL.
Controls which episodes automation should look for.|Détermine quels épisodes l'automatisation doit rechercher.|Bestimmt, welche Episoden die Automatisierung sucht.|Determina quali episodi deve cercare l'automazione.|Determina os episódios que a automatização deve pesquisar.
Create global qBittorrent tags for manual organization.|Créer des étiquettes globales qBittorrent pour l'organisation manuelle.|Globale qBittorrent-Tags zur manuellen Organisation erstellen.|Crea etichette globali qBittorrent per organizzare manualmente.|Criar etiquetas globais qBittorrent para organização manual.
Direct integration for your multimedia app or other request clients.|Intégration directe pour votre application multimédia ou clients de demandes.|Direkte Integration für Ihre Medien-App oder andere Anfrage-Clients.|Integrazione diretta per l'app multimediale o altri client di richieste.|Integração direta para a aplicação multimédia ou outros clientes de pedidos.
Discover existing media files and rebuild real availability.|Détecter les fichiers existants et actualiser la disponibilité réelle.|Vorhandene Mediendateien erkennen und Verfügbarkeit aktualisieren.|Rileva i file esistenti e aggiorna la disponibilità reale.|Detetar ficheiros existentes e atualizar a disponibilidade real.
Each category can have its own default save path.|Chaque catégorie peut avoir son dossier par défaut.|Jede Kategorie kann einen eigenen Standard-Speicherpfad haben.|Ogni categoria può avere il proprio percorso predefinito.|Cada categoria pode ter o seu caminho predefinido.
Each movie or series can use a different profile.|Chaque film ou série peut utiliser un profil différent.|Jeder Film und jede Serie kann ein anderes Profil verwenden.|Ogni film o serie può usare un profilo diverso.|Cada filme ou série pode usar um perfil diferente.
Future monitored episodes will appear here.|Les épisodes futurs suivis apparaîtront ici.|Zukünftige überwachte Episoden erscheinen hier.|Gli episodi futuri monitorati appariranno qui.|Os episódios futuros monitorizados aparecerão aqui.
If Oberiz is useful to you, you can support its development through PayPal.|Si Oberiz vous est utile, vous pouvez soutenir son développement via PayPal.|Wenn Oberiz Ihnen hilft, können Sie die Entwicklung über PayPal unterstützen.|Se Oberiz ti è utile, puoi sostenerne lo sviluppo tramite PayPal.|Se o Oberiz lhe é útil, pode apoiar o desenvolvimento via PayPal.
Imports, searches and downloads will be recorded here.|Les importations, recherches et téléchargements seront enregistrés ici.|Importe, Suchen und Downloads werden hier aufgezeichnet.|Importazioni, ricerche e download saranno registrati qui.|Importações, pesquisas e transferências serão registadas aqui.
Independent movie, series and language rules for matching, scoring and automatic downloads.|Règles indépendantes pour films, séries et langues : correspondance, scores et téléchargements automatiques.|Unabhängige Film-, Serien- und Sprachregeln für Zuordnung, Bewertung und automatische Downloads.|Regole indipendenti per film, serie e lingue: corrispondenze, punteggi e download automatici.|Regras independentes para filmes, séries e idiomas: correspondências, pontuações e transferências automáticas.
Live torrent activity from qBittorrent.|Activité des torrents qBittorrent en temps réel.|Torrent-Aktivität von qBittorrent in Echtzeit.|Attività torrent di qBittorrent in tempo reale.|Atividade de torrents qBittorrent em tempo real.
Lower priority runs first and breaks ties between equally scored releases. The custom tag replaces|La priorité la plus basse passe en premier et départage les scores égaux. L'étiquette personnalisée remplace|Die niedrigere Priorität kommt zuerst und entscheidet bei gleicher Punktzahl. Das eigene Tag ersetzt|La priorità più bassa viene prima e risolve le parità di punteggio. L'etichetta personalizzata sostituisce|A prioridade mais baixa vem primeiro e desempata pontuações iguais. A etiqueta personalizada substitui
Media, downloads, reseed workspace and metadata.|Contenu, téléchargements, dossier de reseed et métadonnées.|Medien, Downloads, Reseed-Verzeichnis und Metadaten.|Contenuti, download, cartella reseed e metadati.|Conteúdo, transferências, pasta de reseed e metadados.
Never pick this release again for this title|Ne plus choisir cette version pour ce titre|Diese Veröffentlichung nie wieder für diesen Titel wählen|Non scegliere più questa versione per questo titolo|Nunca voltar a escolher esta versão para este título
New grabs will appear here with their live progress.|Les nouveaux téléchargements apparaîtront ici avec leur progression.|Neue Downloads erscheinen hier mit ihrem aktuellen Fortschritt.|I nuovi download appariranno qui con il progresso in tempo reale.|As novas transferências aparecerão aqui com o progresso em tempo real.
No Oberiz import jobs yet.|Aucune tâche d'importation pour le moment.|Noch keine Oberiz-Importaufträge.|Nessuna attività di importazione ancora.|Ainda sem tarefas de importação.
No RSS synchronization has run yet.|Aucune synchronisation RSS effectuée.|Noch keine RSS-Synchronisierung ausgeführt.|Nessuna sincronizzazione RSS eseguita.|Ainda não foi executada uma sincronização RSS.
No backups created yet.|Aucune sauvegarde créée.|Noch keine Sicherungen erstellt.|Nessun backup creato.|Ainda sem cópias de segurança.
No categories configured in qBittorrent.|Aucune catégorie configurée dans qBittorrent.|Keine Kategorien in qBittorrent konfiguriert.|Nessuna categoria configurata in qBittorrent.|Sem categorias configuradas no qBittorrent.
No completed torrents yet.|Aucun torrent terminé.|Noch keine abgeschlossenen Torrents.|Nessun torrent completato.|Ainda sem torrents concluídos.
No episodes in this calendar window.|Aucun épisode dans cette période.|Keine Episoden in diesem Zeitraum.|Nessun episodio in questo intervallo.|Sem episódios neste intervalo.
No events yet.|Aucun événement pour le moment.|Noch keine Ereignisse.|Nessun evento ancora.|Ainda sem eventos.
No indexers configured yet. Use|Aucun indexeur configuré. Utilisez|Noch keine Indexer konfiguriert. Verwenden Sie|Nessun indicizzatore configurato. Usa|Sem indexadores configurados. Use
No movies match the current filters.|Aucun film ne correspond aux filtres.|Keine Filme passen zu den aktuellen Filtern.|Nessun film corrisponde ai filtri.|Nenhum filme corresponde aos filtros atuais.
No movies or series in your library match “|Aucun film ou série ne correspond à «|Kein Film und keine Serie passt zu „|Nessun film o serie corrisponde a «|Nenhum filme ou série corresponde a «
No recent errors recorded.|Aucune erreur récente enregistrée.|Keine aktuellen Fehler aufgezeichnet.|Nessun errore recente registrato.|Sem erros recentes registados.
No series match the current filters.|Aucune série ne correspond aux filtres.|Keine Serien passen zu den aktuellen Filtern.|Nessuna serie corrisponde ai filtri.|Nenhuma série corresponde aos filtros atuais.
No tags configured in qBittorrent.|Aucune étiquette configurée dans qBittorrent.|Keine Tags in qBittorrent konfiguriert.|Nessuna etichetta configurata in qBittorrent.|Sem etiquetas configuradas no qBittorrent.
Oberiz is free to use. Donations are entirely optional.|Oberiz est gratuit. Les dons sont entièrement facultatifs.|Oberiz ist kostenlos. Spenden sind freiwillig.|Oberiz è gratuito. Le donazioni sono facoltative.|O Oberiz é gratuito. Os donativos são opcionais.
Oberiz stores only its hash, not the key itself — once you leave or reload this page it can't be shown again. Generate a new one if you lose it.|Oberiz ne stocke que l'empreinte de la clé. Après avoir quitté ou rechargé cette page, elle ne sera plus visible. Générez-en une autre si vous la perdez.|Oberiz speichert nur den Hash des Schlüssels. Nach Verlassen oder Neuladen der Seite ist er nicht mehr sichtbar. Bei Verlust einen neuen erzeugen.|Oberiz salva solo l'hash della chiave. Dopo aver lasciato o ricaricato la pagina non sarà più visibile. Generane un'altra se la perdi.|O Oberiz guarda apenas o hash da chave. Depois de sair ou recarregar a página, não poderá vê-la novamente. Gere outra se a perder.
Oberiz will not pick these again for the title shown. Undo one to let it be chosen again.|Oberiz ne choisira plus ces versions pour ce titre. Annulez le rejet pour les autoriser.|Oberiz wählt diese Veröffentlichungen nicht erneut für den Titel. Ablehnung aufheben, um sie wieder zuzulassen.|Oberiz non sceglierà più queste versioni per il titolo. Annulla il rifiuto per consentirle.|O Oberiz não escolherá estas versões novamente para este título. Desfaça a rejeição para as permitir.
Only configured indexers appear here.|Seuls les indexeurs configurés apparaissent ici.|Hier erscheinen nur konfigurierte Indexer.|Qui appaiono solo gli indicizzatori configurati.|Aqui aparecem apenas indexadores configurados.
Only needed when qBittorrent runs on a different machine than Oberiz. Leave both blank if they share the same filesystem.|Nécessaire seulement si qBittorrent est sur un autre ordinateur. Laissez les deux champs vides s'ils partagent le système de fichiers.|Nur nötig, wenn qBittorrent auf einem anderen Rechner läuft. Beide leer lassen, wenn dasselbe Dateisystem verwendet wird.|Necessario solo se qBittorrent è su un altro computer. Lascia entrambi vuoti se condividono il filesystem.|Só necessário se o qBittorrent estiver noutro computador. Deixe ambos vazios se partilharem o sistema de ficheiros.
Optional series identifier lookup for compatible indexers|Recherche facultative d'identifiants de séries pour indexeurs compatibles|Optionale Suche nach Serienkennungen für kompatible Indexer|Ricerca facoltativa di identificatori serie per indicizzatori compatibili|Pesquisa opcional de identificadores de séries para indexadores compatíveis
Per-indexer cleanup rules. No policy means no automatic deletion.|Règles de nettoyage par indexeur. Sans règle, aucune suppression automatique.|Bereinigungsregeln je Indexer. Ohne Richtlinie keine automatische Löschung.|Regole di pulizia per indicizzatore. Senza regola non si elimina automaticamente.|Regras de limpeza por indexador. Sem política não há eliminação automática.
Process only new feed releases from configured indexers; each release is deduplicated before matching.|Traiter seulement les nouvelles versions des flux configurés ; les doublons sont éliminés avant association.|Nur neue Veröffentlichungen konfigurierter Feeds verarbeiten; Duplikate werden vor der Zuordnung entfernt.|Elabora solo le nuove versioni dei feed configurati; i duplicati sono rimossi prima dell'associazione.|Processar apenas novas versões dos feeds configurados; os duplicados são removidos antes da associação.
Protects the whole Oberiz interface and API when it is reachable on your network.|Protège l'interface et l'API d'Oberiz accessibles sur votre réseau.|Schützt die gesamte Oberiz-Oberfläche und API im Netzwerk.|Protegge l'interfaccia e l'API di Oberiz accessibili in rete.|Protege toda a interface e API do Oberiz acessíveis na rede.
Read-only status for support and troubleshooting. Secrets are never included.|État en lecture seule pour diagnostic et assistance. Aucun secret n'est inclus.|Schreibgeschützter Status für Support und Diagnose. Zugangsdaten werden nie enthalten.|Stato in sola lettura per assistenza e diagnostica. Non include credenziali.|Estado só de leitura para suporte e diagnóstico. Nunca inclui credenciais.
Real events recorded by Oberiz.|Événements réels enregistrés par Oberiz.|Von Oberiz aufgezeichnete Ereignisse.|Eventi reali registrati da Oberiz.|Eventos reais registados pelo Oberiz.
Reject this release: remove it and never pick it again|Rejeter cette version : la supprimer et ne plus la choisir|Diese Veröffentlichung ablehnen: entfernen und nie wieder wählen|Rifiuta questa versione: rimuovila e non sceglierla più|Rejeitar esta versão: removê-la e nunca voltar a escolhê-la
Required to use Oberiz. It keeps you signed in on this browser for 30 days; changing or removing it needs the current password.|Requis pour utiliser Oberiz. La session reste ouverte 30 jours dans ce navigateur ; modifier ou supprimer le mot de passe nécessite l'actuel.|Für Oberiz erforderlich. Die Anmeldung bleibt 30 Tage in diesem Browser aktiv; Ändern oder Entfernen erfordert das aktuelle Passwort.|Necessaria per usare Oberiz. Mantiene l'accesso per 30 giorni; modificarla o rimuoverla richiede la password attuale.|Necessária para usar o Oberiz. Mantém a sessão durante 30 dias; alterá-la ou removê-la requer a palavra-passe atual.
Rescan does not move or rename files. It indexes files already present in the configured library paths, refreshes Movies availability and links SxxExx files to Series episodes.|L'analyse ne déplace ni renomme les fichiers. Elle indexe les fichiers existants, actualise la disponibilité des films et associe les fichiers SxxExx aux épisodes.|Einlesen verschiebt oder benennt keine Dateien um. Vorhandene Dateien werden indexiert, Filmverfügbarkeit aktualisiert und SxxExx-Dateien Episoden zugeordnet.|La scansione non sposta né rinomina file. Indicizza i file esistenti, aggiorna la disponibilità dei film e associa SxxExx agli episodi.|A análise não move nem muda nomes de ficheiros. Indexa os existentes, atualiza a disponibilidade dos filmes e associa SxxExx aos episódios.
Runtime tests and failures will appear here.|Les tests et erreurs d'exécution apparaîtront ici.|Laufzeittests und Fehler erscheinen hier.|Verifiche ed errori di esecuzione appariranno qui.|Os testes e erros de execução aparecerão aqui.
Set a password once to protect this Oberiz instance before using it.|Définissez un mot de passe pour protéger cette instance avant utilisation.|Vor der Nutzung ein Passwort zum Schutz dieser Instanz festlegen.|Imposta una password per proteggere questa istanza prima dell'uso.|Defina uma palavra-passe para proteger esta instância antes de a usar.
TMDB is configured independently by each user with their own API credential and subject to TMDB’s terms.|Chaque utilisateur configure TMDB avec sa propre clé API, conformément aux conditions de TMDB.|Jeder Nutzer konfiguriert TMDB mit eigenen API-Zugangsdaten gemäß den TMDB-Bedingungen.|Ogni utente configura TMDB con le proprie credenziali API secondo le condizioni di TMDB.|Cada utilizador configura o TMDB com as suas credenciais API e de acordo com as condições do TMDB.
The profile controls quality, language, upgrades and qBittorrent routing.|Le profil contrôle qualité, langue, améliorations et destination qBittorrent.|Das Profil steuert Qualität, Sprache, Verbesserungen und qBittorrent-Ziel.|Il profilo controlla qualità, lingua, miglioramenti e destinazione qBittorrent.|O perfil controla qualidade, idioma, melhorias e destino qBittorrent.
The torrent is removed from qBittorrent and Oberiz will never pick this release again for this title; its next search will look for another one. You can undo it from the manual search.|Le torrent est supprimé de qBittorrent et cette version ne sera plus choisie pour ce titre. La prochaine recherche en cherchera une autre. Vous pouvez annuler depuis la recherche manuelle.|Der Torrent wird aus qBittorrent entfernt und nie wieder für diesen Titel gewählt. Die nächste Suche sucht eine andere Veröffentlichung. In der manuellen Suche rückgängig machen.|Il torrent viene rimosso da qBittorrent e non sarà più scelto per questo titolo. La prossima ricerca ne cercherà un altro. Puoi annullare dalla ricerca manuale.|O torrent é removido do qBittorrent e não voltará a ser escolhido para este título. A próxima pesquisa procurará outro. Pode desfazer na pesquisa manual.
This Oberiz instance is protected with an admin password.|Cette instance est protégée par un mot de passe administrateur.|Diese Instanz ist mit einem Administratorpasswort geschützt.|Questa istanza è protetta da una password amministratore.|Esta instância está protegida por palavra-passe de administrador.
This permanently removes the data from disk.|Cela supprime définitivement les données du disque.|Dies löscht die Daten dauerhaft vom Datenträger.|Elimina definitivamente i dati dal disco.|Isto elimina permanentemente os dados do disco.
This record has no reusable .torrent metadata and library path.|Cet enregistrement n'a ni métadonnées .torrent réutilisables ni chemin de bibliothèque.|Dieser Eintrag hat keine wiederverwendbaren .torrent-Metadaten und keinen Bibliothekspfad.|Questo record non ha metadati .torrent riutilizzabili né percorso libreria.|Este registo não tem metadados .torrent reutilizáveis nem caminho de biblioteca.
Track seasons and episodes with inherited profiles and monitoring rules|Suivez saisons et épisodes avec profils et règles hérités|Staffeln und Episoden mit geerbten Profilen und Regeln überwachen|Gestisci stagioni ed episodi con profili e regole ereditati|Monitorizar temporadas e episódios com perfis e regras herdados
Upcoming and recently aired episodes from your Oberiz library.|Épisodes à venir et récemment diffusés de votre bibliothèque.|Kommende und kürzlich ausgestrahlte Episoden Ihrer Bibliothek.|Episodi in arrivo e appena trasmessi dalla tua libreria.|Episódios próximos e recentemente emitidos da biblioteca.
Use the same key in both services. The movie and series root folders and quality profiles come from Oberiz.|Utilisez la même clé dans les deux services. Les dossiers racine et profils de qualité viennent d'Oberiz.|Denselben Schlüssel für beide Dienste verwenden. Stammordner und Qualitätsprofile kommen aus Oberiz.|Usa la stessa chiave nei due servizi. Cartelle principali e profili qualità provengono da Oberiz.|Use a mesma chave nos dois serviços. As pastas raiz e perfis de qualidade vêm do Oberiz.
Use this indexer in release searches.|Utiliser cet indexeur dans les recherches.|Diesen Indexer für die Suche verwenden.|Usa questo indicizzatore nelle ricerche.|Usar este indexador nas pesquisas.
Used only to resolve a TVDB series ID for indexers that request it. It is never sent to indexers.|Utilisé seulement pour obtenir l'ID TVDB demandé par les indexeurs. La clé ne leur est jamais envoyée.|Nur zur Ermittlung einer TVDB-Serien-ID für Indexer verwendet. Der Schlüssel wird nie an Indexer gesendet.|Usata solo per ottenere l'ID TVDB richiesto dagli indicizzatori. La chiave non viene mai inviata agli indicizzatori.|Usada apenas para obter o ID TVDB solicitado por indexadores. A chave nunca lhes é enviada.
can be removed safely. Use Shift to select a range.|peuvent être supprimés. Utilisez Maj pour sélectionner une plage.|können entfernt werden. Mit Umschalt einen Bereich auswählen.|possono essere rimossi. Usa Maiusc per selezionare un intervallo.|podem ser removidos. Use Shift para selecionar um intervalo.
configured feeds · last interval:|flux configurés · dernier intervalle :|konfigurierte Feeds · letztes Intervall:|feed configurati · ultimo intervallo:|feeds configurados · último intervalo:
days · ratio|jours · ratio|Tage · Verhältnis|giorni · rapporto|dias · rácio
in each Indexer configuration. Existing releases are remembered, so a feed item is evaluated once only.|dans chaque indexeur. Les versions existantes sont mémorisées pour n'évaluer chaque élément qu'une fois.|in jedem Indexer. Vorhandene Veröffentlichungen werden gespeichert, damit jedes Feed-Element nur einmal geprüft wird.|in ogni indicizzatore. Le versioni esistenti sono memorizzate per valutare ogni elemento una sola volta.|em cada indexador. As versões existentes são memorizadas para avaliar cada item apenas uma vez.
in the selected quality profile’s tag template. "Only use RSS" still lets this indexer be searched when you add a title or click Search — it's skipped only by the scheduled automatic search.|dans le modèle d'étiquettes du profil. « RSS uniquement » permet toujours la recherche lors de l'ajout d'un titre ou avec Rechercher ; seule la recherche automatique planifiée l'ignore.|in der Tag-Vorlage des Profils. „Nur RSS“ erlaubt weiterhin die Suche beim Hinzufügen eines Titels oder über Suchen; nur die geplante automatische Suche überspringt ihn.|nel modello etichette del profilo. «Solo RSS» consente la ricerca aggiungendo un titolo o con Cerca; viene saltata solo la ricerca automatica pianificata.|no modelo de etiquetas do perfil. «Só RSS» permite pesquisar ao adicionar um título ou premir Pesquisar; apenas a pesquisa automática agendada o ignora.
new ·|nouveaux ·|neu ·|nuovi ·|novos ·
to choose one from the definition catalogue.|pour en choisir un dans le catalogue.|um einen aus dem Definitionskatalog zu wählen.|per sceglierne uno dal catalogo.|para escolher um do catálogo.
→ qBittorrent gets the destination category and tags such as|→ qBittorrent reçoit la catégorie et des étiquettes telles que|→ qBittorrent erhält die Zielkategorie und Tags wie|→ qBittorrent riceve la categoria e le etichette come|→ qBittorrent recebe a categoria e etiquetas como
API key saved. Copy it now — it will not be shown again.|Clé API enregistrée. Copiez-la maintenant ; elle ne sera plus affichée.|API-Schlüssel gespeichert. Jetzt kopieren; er wird nicht erneut angezeigt.|Chiave API salvata. Copiala adesso; non sarà più mostrata.|Chave API guardada. Copie-a agora; não será mostrada novamente.
Admin password saved. Use it next time Oberiz asks you to sign in.|Mot de passe enregistré. Utilisez-le lors de la prochaine connexion.|Passwort gespeichert. Bei der nächsten Anmeldung verwenden.|Password salvata. Usala al prossimo accesso.|Palavra-passe guardada. Use-a no próximo início de sessão.
Choose another default profile before deleting|Choisissez un autre profil par défaut avant de supprimer|Vor dem Löschen ein anderes Standardprofil wählen|Scegli un altro profilo predefinito prima di eliminare|Escolha outro perfil predefinido antes de eliminar
Choose how Oberiz should manage this title.|Choisissez comment Oberiz doit gérer ce titre.|Wählen Sie, wie Oberiz diesen Titel verwalten soll.|Scegli come Oberiz deve gestire questo titolo.|Escolha como o Oberiz deve gerir este título.
Could not access clipboard. Select and copy the key manually.|Impossible d'accéder au presse-papiers. Copiez la clé manuellement.|Kein Zugriff auf die Zwischenablage. Schlüssel manuell kopieren.|Impossibile accedere agli appunti. Copia la chiave manualmente.|Não foi possível aceder à área de transferência. Copie a chave manualmente.
Loaded directly from qBittorrent. Empty means truly uncategorized; save paths remain managed by qBittorrent.|Chargé depuis qBittorrent. Vide signifie sans catégorie ; qBittorrent gère les dossiers.|Direkt aus qBittorrent geladen. Leer bedeutet ohne Kategorie; Speicherpfade verwaltet qBittorrent.|Caricato da qBittorrent. Vuoto significa senza categoria; qBittorrent gestisce i percorsi.|Carregado do qBittorrent. Vazio significa sem categoria; qBittorrent gere os caminhos.
New key generated. Save Settings to activate it.|Nouvelle clé générée. Enregistrez les paramètres pour l'activer.|Neuer Schlüssel erzeugt. Einstellungen speichern, um ihn zu aktivieren.|Nuova chiave generata. Salva le impostazioni per attivarla.|Nova chave gerada. Guarde as definições para a ativar.
New key generated. Save Settings to replace the current key.|Nouvelle clé générée. Enregistrez pour remplacer l'actuelle.|Neuer Schlüssel erzeugt. Einstellungen speichern, um den aktuellen zu ersetzen.|Nuova chiave generata. Salva per sostituire quella attuale.|Nova chave gerada. Guarde para substituir a atual.
Remove the admin password? Oberiz will ask you to create a new one the next time it's opened; there is no way to leave it unprotected.|Supprimer le mot de passe administrateur ? Oberiz demandera d'en créer un nouveau à la prochaine ouverture ; il ne peut pas rester sans protection.|Administratorpasswort entfernen? Oberiz fordert beim nächsten Öffnen ein neues an; es kann nicht ungeschützt bleiben.|Rimuovere la password amministratore? Oberiz ne richiederà una nuova alla prossima apertura; non può restare senza protezione.|Remover a palavra-passe de administrador? O Oberiz pedirá uma nova ao abrir; não pode ficar sem proteção.
Select for removal — hold Shift to select a range|Sélectionner pour supprimer ; maintenir Maj pour une plage|Zum Entfernen auswählen; mit Umschalt einen Bereich markieren|Seleziona per rimuovere; tieni Maiusc per un intervallo|Selecionar para remover; mantenha Shift para um intervalo
qBittorrent is connected. Add a magnet or connect your main qBittorrent later.|qBittorrent est connecté. Ajoutez un magnet ou connectez votre qBittorrent principal plus tard.|qBittorrent ist verbunden. Magnet hinzufügen oder später den Hauptclient verbinden.|qBittorrent è connesso. Aggiungi un magnet o collega il client principale più tardi.|O qBittorrent está ligado. Adicione um magnet ou ligue o cliente principal mais tarde.
Settings|Paramètres|Einstellungen|Impostazioni|Definições
Indexers|Indexeurs|Indexer|Indicizzatori|Indexadores
Downloads|Téléchargements|Downloads|Download|Transferências
Movies|Films|Filme|Film|Filmes
Series|Séries|Serien|Serie|Séries
Profiles|Profils|Profile|Profili|Perfis
Appearance|Apparence|Darstellung|Aspetto|Aparência
Theme|Thème|Design|Tema|Tema
Dark|Sombre|Dunkel|Scuro|Escuro
Middle|Intermédiaire|Mittel|Intermedio|Intermédio
Light|Clair|Hell|Chiaro|Claro
Add Media|Ajouter du contenu|Medien hinzufügen|Aggiungi contenuti|Adicionar conteúdo
History|Historique|Verlauf|Cronologia|Histórico
Calendar|Calendrier|Kalender|Calendario|Calendário
Imports & Reseed|Importation et reseed|Import und Reseed|Importazione e reseed|Importação e reseed
Notifications & activity|Notifications et activité|Benachrichtigungen und Aktivität|Notifiche e attività|Notificações e atividade
Search Oberiz...|Rechercher dans Oberiz…|Oberiz durchsuchen…|Cerca in Oberiz…|Pesquisar no Oberiz…
Portal language|Langue de l'interface|Sprache der Oberfläche|Lingua dell'interfaccia|Idioma da interface
Status unknown|État inconnu|Status unbekannt|Stato sconosciuto|Estado desconhecido
Needs attention|Nécessite une intervention|Handlungsbedarf|Richiede attenzione|Requer atenção
All Systems Operational|Tous les systèmes fonctionnent|Alle Systeme betriebsbereit|Tutti i sistemi funzionanti|Todos os sistemas operacionais
Backup created successfully.|Sauvegarde créée.|Sicherung erstellt.|Backup creato.|Cópia de segurança criada.
Backup uploaded successfully.|Sauvegarde téléversée.|Sicherung hochgeladen.|Backup caricato.|Cópia de segurança enviada.
Backup deleted.|Sauvegarde supprimée.|Sicherung gelöscht.|Backup eliminato.|Cópia de segurança eliminada.
Backup restored successfully.|Sauvegarde restaurée.|Sicherung wiederhergestellt.|Backup ripristinato.|Cópia de segurança restaurada.
Connecting to Oberiz…|Connexion à Oberiz…|Verbindung mit Oberiz…|Connessione a Oberiz…|A ligar ao Oberiz…
Search priority|Priorité de recherche|Suchpriorität|Priorità di ricerca|Prioridade de pesquisa
Custom qBittorrent tag (optional)|Étiquette qBittorrent personnalisée (facultatif)|Benutzerdefiniertes qBittorrent-Tag (optional)|Etichetta qBittorrent personalizzata (facoltativa)|Etiqueta qBittorrent personalizada (opcional)
RSS feed URL (optional)|URL du flux RSS (facultatif)|RSS-Feed-URL (optional)|URL del feed RSS (facoltativo)|URL do feed RSS (opcional)
Search priority must be a whole number|La priorité doit être un entier|Die Priorität muss eine ganze Zahl sein|La priorità deve essere un numero intero|A prioridade deve ser um número inteiro
Search priority must be between 0 and 10000|La priorité doit être comprise entre 0 et 10000|Die Priorität muss zwischen 0 und 10000 liegen|La priorità deve essere tra 0 e 10000|A prioridade deve estar entre 0 e 10000
Password protection is not enabled|La protection par mot de passe est désactivée|Passwortschutz ist nicht aktiviert|La protezione con password non è attiva|A proteção por palavra-passe não está ativa
Password is too long|Le mot de passe est trop long|Das Passwort ist zu lang|La password è troppo lunga|A palavra-passe é demasiado longa
Incorrect password|Mot de passe incorrect|Falsches Passwort|Password errata|Palavra-passe incorreta
Current password is too long|Le mot de passe actuel est trop long|Das aktuelle Passwort ist zu lang|La password attuale è troppo lunga|A palavra-passe atual é demasiado longa
Current password is incorrect|Le mot de passe actuel est incorrect|Das aktuelle Passwort ist falsch|La password attuale è errata|A palavra-passe atual é incorreta
Password must be between 8 and 128 characters|Le mot de passe doit comporter de 8 à 128 caractères|Das Passwort muss 8 bis 128 Zeichen lang sein|La password deve avere da 8 a 128 caratteri|A palavra-passe deve ter entre 8 e 128 caracteres
Cleaned|Nettoyé|Bereinigt|Pulito|Limpo
Imported|Importé|Importiert|Importato|Importado
Importing|Importation|Wird importiert|Importazione|A importar
Missing|Manquant|Fehlend|Mancante|Em falta
Duplicate|Doublon|Duplikat|Duplicato|Duplicado
Reseeded|Reseed lancé|Reseed gestartet|Reseed avviato|Reseed iniciado
Reconciled|Récupéré|Abgeglichen|Recuperato|Recuperado
Downloading Metadata|Téléchargement des métadonnées|Metadaten werden geladen|Download dei metadati|A transferir metadados
Pending|En attente|Ausstehend|In attesa|Pendente
Copying|Copie|Wird kopiert|Copia in corso|A copiar
Hardlinking|Création du lien physique|Hardlink wird erstellt|Creazione del collegamento fisico|A criar ligação física
Paused|En pause|Pausiert|In pausa|Em pausa
Running|En cours|Wird ausgeführt|In esecuzione|Em execução
Never monitored|Non suivi|Nicht überwacht|Non monitorato|Não monitorizado
Events|Événements|Ereignisse|Eventi|Eventos
Add Oberiz twice under|Ajoutez Oberiz deux fois dans|Oberiz zweimal hinzufügen unter|Aggiungi Oberiz due volte in|Adicione o Oberiz duas vezes em
· job #|· tâche nº|· Auftrag Nr.|· attività n.|· tarefa n.º
Resolutions|Résolutions|Auflösungen|Risoluzioni|Resoluções
Sources|Sources|Quellen|Sorgenti|Fontes
Codecs|Codecs|Codecs|Codec|Codecs
Hdr|HDR|HDR|HDR|HDR
Audio|Audio|Audio|Audio|Áudio
Configured|Configuré|Konfiguriert|Configurato|Configurado
Disabled|Désactivé|Deaktiviert|Disattivato|Desativado
Healthy|Bon état|In Ordnung|Regolare|Em bom estado
Update|Mettre à jour|Aktualisieren|Aggiorna|Atualizar
Planned|Planifié|Geplant|Pianificato|Planeado
Downloading|Téléchargement|Wird heruntergeladen|Download in corso|A transferir
Completed|Terminé|Abgeschlossen|Completato|Concluído
Error|Erreur|Fehler|Errore|Erro
Rejected|Rejeté|Abgelehnt|Rifiutato|Rejeitado
Queued|En file d'attente|In Warteschlange|In coda|Em fila
Copy|Copier|Kopieren|Copia|Copiar
Hardlink|Lien physique|Hardlink|Collegamento fisico|Ligação física
Move|Déplacer|Verschieben|Sposta|Mover
Browse|Parcourir|Durchsuchen|Sfoglia|Procurar
All|Tous|Alle|Tutti|Todos
Offline|Hors ligne|Offline|Offline|Offline
Spanish preferred|Espagnol préféré|Spanisch bevorzugt|Spagnolo preferito|Espanhol preferido
English preferred|Anglais préféré|Englisch bevorzugt|Inglese preferito|Inglês preferido
Languages|Langues|Sprachen|Lingue|Idiomas
Load more|Charger plus|Mehr laden|Carica altro|Carregar mais
Min|Minimum|Minimum|Minimo|Mínimo
Max|Maximum|Maximum|Massimo|Máximo
Total|Total|Gesamt|Totale|Total
Add {0}|Ajouter {0}|{0} hinzufügen|Aggiungi {0}|Adicionar {0}
Delete {0}?|Supprimer {0} ?|{0} löschen?|Eliminare {0}?|Eliminar {0}?
Downloading {0}%|Téléchargement {0}%|Download {0}%|Download {0}%|A transferir {0}%
Movie · {0}|Film · {0}|Film · {0}|Film · {0}|Filme · {0}
Remove selected{0}|Supprimer la sélection{0}|Auswahl entfernen{0}|Rimuovi selezionati{0}|Remover selecionados{0}
Remove {0}|Supprimer {0}|{0} entfernen|Rimuovi {0}|Remover {0}
Removed {0}|{0} supprimé|{0} entfernt|{0} rimosso|{0} removido
Reseed started in {0}|Reseed lancé dans {0}|Reseed in {0} gestartet|Reseed avviato in {0}|Reseed iniciado em {0}
Running · {0}/{1} · {2}|En cours · {0}/{1} · {2}|Läuft · {0}/{1} · {2}|In esecuzione · {0}/{1} · {2}|Em execução · {0}/{1} · {2}
Search {0}...|Rechercher {0}…|{0} suchen…|Cerca {0}…|Pesquisar {0}…
Season {0}|Saison {0}|Staffel {0}|Stagione {0}|Temporada {0}
Series · {0}|Série · {0}|Serie · {0}|Serie · {0}|Série · {0}
Updated {0}|Mis à jour {0}|Aktualisiert {0}|Aggiornato {0}|Atualizado {0}
every {0} min|toutes les {0} min|alle {0} Min.|ogni {0} min|a cada {0} min
{0} / {1} enabled|{0} / {1} activés|{0} / {1} aktiviert|{0} / {1} attivi|{0} / {1} ativos
{0} Copy|{0} Copie|{0} Kopie|{0} Copia|{0} Cópia
{0} Reloading…|{0} Rechargement…|{0} Wird neu geladen…|{0} Ricaricamento…|{0} A recarregar…
{0} Season {1}|{0} Saison {1}|{0} Staffel {1}|{0} Stagione {1}|{0} Temporada {1}
{0} d ago|il y a {0} j|vor {0} Tagen|{0} giorni fa|há {0} dias
{0} days ago|il y a {0} jours|vor {0} Tagen|{0} giorni fa|há {0} dias
{0} episodes|{0} épisodes|{0} Episoden|{0} episodi|{0} episódios
{0} feed{1} · {2}|{0} flux · {2}|{0} Feeds · {2}|{0} feed · {2}|{0} feeds · {2}
{0} h ago|il y a {0} h|vor {0} Std.|{0} ore fa|há {0} h
{0} historical issue{1}|{0} incidents historiques|{0} frühere Probleme|{0} problemi precedenti|{0} problemas anteriores
{0} hours ago|il y a {0} heures|vor {0} Stunden|{0} ore fa|há {0} horas
{0} min ago|il y a {0} min|vor {0} Min.|{0} min fa|há {0} min
{0} minutes ago|il y a {0} minutes|vor {0} Minuten|{0} minuti fa|há {0} minutos
{0} missing episode{1}|{0} épisodes manquants|{0} fehlende Episoden|{0} episodi mancanti|{0} episódios em falta
{0} record{1} removed.|{0} enregistrements supprimés.|{0} Einträge entfernt.|{0} record rimossi.|{0} registos removidos.
{0} season{1}|{0} saisons|{0} Staffeln|{0} stagioni|{0} temporadas
{0} · ETA {1} · {2} seeds|{0} · reste {1} · {2} sources|{0} · noch {1} · {2} Seeds|{0} · mancano {1} · {2} seed|{0} · faltam {1} · {2} sementes
{0} · {1} queued|{0} · {1} en attente|{0} · {1} in Warteschlange|{0} · {1} in coda|{0} · {1} em fila
{0}% average|{0}% en moyenne|{0}% im Mittel|{0}% in media|{0}% em média
{0}% covered|{0}% couvert|{0}% abgedeckt|{0}% coperto|{0}% coberto
{0}% healthy|{0}% en bon état|{0}% in Ordnung|{0}% regolare|{0}% em bom estado
{0}/{1} available|{0}/{1} disponibles|{0}/{1} verfügbar|{0}/{1} disponibili|{0}/{1} disponíveis
{0}/{1} processed|{0}/{1} traités|{0}/{1} verarbeitet|{0}/{1} elaborati|{0}/{1} processados
{0}: {1}/{2} matched|{0} : {1}/{2} associés|{0}: {1}/{2} zugeordnet|{0}: {1}/{2} associati|{0}: {1}/{2} associados
{0}h {1}m left|reste {0} h {1} min|noch {0} Std. {1} Min.|mancano {0} h {1} min|faltam {0} h {1} min
{0}m left|reste {0} min|noch {0} Min.|mancano {0} min|faltam {0} min
% of library|% de la bibliothèque|% der Bibliothek|% della libreria|% da biblioteca
(optional)|(facultatif)|(optional)|(facoltativo)|(opcional)
+ tags|+ étiquettes|+ Tags|+ etichette|+ etiquetas
, using the API key above:|, avec la clé API ci-dessus :|, mit dem obigen API-Schlüssel:|, usando la chiave API sopra:|, usando a chave API acima:
14 days|14 jours|14 Tage|14 giorni|14 dias
30 days|30 jours|30 Tage|30 giorni|30 dias
45 days|45 jours|45 Tage|45 giorni|45 dias
90 days|90 jours|90 Tage|90 giorni|90 dias
API key|Clé API|API-Schlüssel|Chiave API|Chave API
Accept complete series|Accepter les séries complètes|Komplette Serien akzeptieren|Accetta serie complete|Aceitar séries completas
Action|Action|Aktion|Azione|Ação
Actions|Actions|Aktionen|Azioni|Ações
Active Downloads|Téléchargements actifs|Aktive Downloads|Download attivi|Transferências ativas
Active Indexers|Indexeurs actifs|Aktive Indexer|Indicizzatori attivi|Indexadores ativos
Activity History|Historique d'activité|Aktivitätsverlauf|Cronologia attività|Histórico de atividade
Add Indexer|Ajouter un indexeur|Indexer hinzufügen|Aggiungi indicizzatore|Adicionar indexador
Add Magnet|Ajouter un magnet|Magnet hinzufügen|Aggiungi magnet|Adicionar magnet
Add to qBittorrent|Ajouter à qBittorrent|Zu qBittorrent hinzufügen|Aggiungi a qBittorrent|Adicionar ao qBittorrent
Added|Ajouté|Hinzugefügt|Aggiunto|Adicionado
Admin Password|Mot de passe administrateur|Administratorpasswort|Password amministratore|Palavra-passe de administrador
Air Date|Date de diffusion|Ausstrahlungsdatum|Data di trasmissione|Data de emissão
All Availability|Toute disponibilité|Jede Verfügbarkeit|Qualsiasi disponibilità|Qualquer disponibilidade
All Episodes|Tous les épisodes|Alle Episoden|Tutti gli episodi|Todos os episódios
All Monitoring|Tous les suivis|Jede Überwachung|Qualsiasi monitoraggio|Qualquer monitorização
All Quality|Toute qualité|Jede Qualität|Qualsiasi qualità|Qualquer qualidade
All Series|Toutes les séries|Alle Serien|Tutte le serie|Todas as séries
Allow unknown resolution|Autoriser une résolution inconnue|Unbekannte Auflösung erlauben|Consenti risoluzione sconosciuta|Permitir resolução desconhecida
Allow unknown source|Autoriser une source inconnue|Unbekannte Quelle erlauben|Consenti sorgente sconosciuta|Permitir fonte desconhecida
Allow upgrades|Autoriser les améliorations|Verbesserungen erlauben|Consenti miglioramenti|Permitir melhorias
Also delete downloaded files|Supprimer aussi les fichiers téléchargés|Auch heruntergeladene Dateien löschen|Elimina anche i file scaricati|Eliminar também os ficheiros transferidos
At least 8 characters|Au moins 8 caractères|Mindestens 8 Zeichen|Almeno 8 caratteri|Pelo menos 8 caracteres
Auto — hardlink, then copy fallback|Auto : lien physique, sinon copie|Automatisch: Hardlink, sonst Kopie|Auto: collegamento fisico, altrimenti copia|Automático: ligação física, senão cópia
Automation|Automatisation|Automatisierung|Automazione|Automatização
Automation scope|Portée de l'automatisation|Automatisierungsumfang|Ambito dell'automazione|Âmbito da automatização
Availability|Disponibilité|Verfügbarkeit|Disponibilità|Disponibilidade
Available|Disponible|Verfügbar|Disponibile|Disponível
Backup & Restore|Sauvegarde et restauration|Sicherung und Wiederherstellung|Backup e ripristino|Cópia de segurança e restauro
Backups are stored in|Les sauvegardes sont stockées dans|Sicherungen werden gespeichert in|I backup sono salvati in|As cópias são guardadas em
Bandwidth Usage|Utilisation de la bande passante|Bandbreitennutzung|Uso della banda|Utilização da largura de banda
Blank = downloads/reseed|Vide = downloads/reseed|Leer = downloads/reseed|Vuoto = downloads/reseed|Vazio = downloads/reseed
Both|Les deux|Beide|Entrambi|Ambos
Build your library|Créez votre bibliothèque|Bibliothek aufbauen|Crea la tua libreria|Crie a sua biblioteca
Cancel|Annuler|Abbrechen|Annulla|Cancelar
Categories|Catégories|Kategorien|Categorie|Categorias
Category|Catégorie|Kategorie|Categoria|Categoria
Category / Tags|Catégorie / Étiquettes|Kategorie / Tags|Categoria / Etichette|Categoria / Etiquetas
Category name|Nom de catégorie|Kategoriename|Nome categoria|Nome da categoria
Cleanup|Nettoyage|Bereinigung|Pulizia|Limpeza
Clients send|Les clients envoient|Clients senden|I client inviano|Os clientes enviam
Clone|Cloner|Klonen|Clona|Clonar
Close|Fermer|Schließen|Chiudi|Fechar
Close menu|Fermer le menu|Menü schließen|Chiudi menu|Fechar menu
Common tokens:|Variables courantes :|Übliche Variablen:|Variabili comuni:|Variáveis comuns:
Completed Today|Terminés aujourd'hui|Heute abgeschlossen|Completati oggi|Concluídos hoje
Configure Overseerr|Configurer Overseerr|Overseerr konfigurieren|Configura Overseerr|Configurar Overseerr
Configured Indexers|Indexeurs configurés|Konfigurierte Indexer|Indicizzatori configurati|Indexadores configurados
Confirm new password|Confirmer le nouveau mot de passe|Neues Passwort bestätigen|Conferma nuova password|Confirmar nova palavra-passe
Confirm password|Confirmer le mot de passe|Passwort bestätigen|Conferma password|Confirmar palavra-passe
Copy this key now.|Copiez cette clé maintenant.|Diesen Schlüssel jetzt kopieren.|Copia questa chiave adesso.|Copie esta chave agora.
Create|Créer|Erstellen|Crea|Criar
Create Backup|Créer une sauvegarde|Sicherung erstellen|Crea backup|Criar cópia de segurança
Create an admin password|Créer un mot de passe administrateur|Administratorpasswort erstellen|Crea password amministratore|Criar palavra-passe de administrador
Credential|Identifiant|Zugangsdaten|Credenziale|Credencial
Current jobs|Tâches actuelles|Aktuelle Aufträge|Attività correnti|Tarefas atuais
Current password|Mot de passe actuel|Aktuelles Passwort|Password attuale|Palavra-passe atual
Custom definitions|Définitions personnalisées|Benutzerdefinierte Definitionen|Definizioni personalizzate|Definições personalizadas
Custom indexers|Indexeurs personnalisés|Benutzerdefinierte Indexer|Indicizzatori personalizzati|Indexadores personalizados
Download client|Client de téléchargement|Download-Client|Client di download|Cliente de transferências
Download routing|Destination des téléchargements|Download-Ziel|Destinazione download|Destino das transferências
Dark theme|Thème sombre|Dunkles Design|Tema scuro|Tema escuro
Default|Par défaut|Standard|Predefinito|Predefinido
Default Series Profile|Profil de séries par défaut|Standardprofil für Serien|Profilo serie predefinito|Perfil de séries predefinido
Default profile|Profil par défaut|Standardprofil|Profilo predefinito|Perfil predefinido
Default qBittorrent Category|Catégorie qBittorrent par défaut|Standardkategorie für qBittorrent|Categoria qBittorrent predefinita|Categoria qBittorrent predefinida
Default save path|Dossier d'enregistrement par défaut|Standard-Speicherpfad|Percorso di salvataggio predefinito|Caminho de gravação predefinido
Definition Sources|Sources des définitions|Definitionsquellen|Fonti delle definizioni|Fontes das definições
Definition catalogue|Catalogue des définitions|Definitionskatalog|Catalogo delle definizioni|Catálogo de definições
Delete|Supprimer|Löschen|Elimina|Eliminar
Delete Series|Supprimer la série|Serie löschen|Elimina serie|Eliminar série
Details/profile|Détails/profil|Details/Profil|Dettagli/profilo|Detalhes/perfil
Diagnostics|Diagnostic|Diagnose|Diagnostica|Diagnóstico
Dismiss|Ignorer|Verwerfen|Ignora|Ignorar
Download|Télécharger|Herunterladen|Scarica|Transferir
Download filters|Filtres de téléchargement|Download-Filter|Filtri download|Filtros de transferências
Download speed|Vitesse de téléchargement|Download-Geschwindigkeit|Velocità di download|Velocidade de transferência
Downloaded|Téléchargé|Heruntergeladen|Scaricato|Transferido
ETA|Temps restant|Restzeit|Tempo rimanente|Tempo restante
Edit|Modifier|Bearbeiten|Modifica|Editar
Either|L'un ou l'autre|Eines von beiden|Uno dei due|Qualquer um
Enable Overseerr compatibility|Activer la compatibilité Overseerr|Overseerr-Kompatibilität aktivieren|Abilita compatibilità Overseerr|Ativar compatibilidade com Overseerr
Enable RSS feed sync|Activer la synchronisation RSS|RSS-Synchronisierung aktivieren|Abilita sincronizzazione RSS|Ativar sincronização RSS
Enable automatic searches|Activer les recherches automatiques|Automatische Suche aktivieren|Abilita ricerche automatiche|Ativar pesquisas automáticas
Enable public request API|Activer l'API publique de demandes|Öffentliche Anfrage-API aktivieren|Abilita API pubblica delle richieste|Ativar API pública de pedidos
Enabled|Activé|Aktiviert|Attivato|Ativado
English|Anglais|Englisch|Inglese|Inglês
Spanish|Espagnol|Spanisch|Spagnolo|Espanhol
Episode|Épisode|Episode|Episodio|Episódio
Episodes Monitored|Épisodes suivis|Überwachte Episoden|Episodi monitorati|Episódios monitorizados
Episodes in range|Épisodes dans la période|Episoden im Zeitraum|Episodi nell'intervallo|Episódios no intervalo
Example: category|Exemple : catégorie|Beispiel: Kategorie|Esempio: categoria|Exemplo: categoria
Existing Episodes|Épisodes existants|Vorhandene Episoden|Episodi esistenti|Episódios existentes
First Season|Première saison|Erste Staffel|Prima stagione|Primeira temporada
Future Episodes|Épisodes futurs|Zukünftige Episoden|Episodi futuri|Episódios futuros
Future monitored episodes|Épisodes futurs suivis|Zukünftige überwachte Episoden|Episodi futuri monitorati|Episódios futuros monitorizados
Go to dashboard|Ouvrir le tableau de bord|Zur Übersicht|Vai alla panoramica|Ir para o painel
Hardlink only|Lien physique uniquement|Nur Hardlink|Solo collegamento fisico|Só ligação física
Host|Hôte|Host|Host|Servidor
Indexer settings|Paramètres de l'indexeur|Indexer-Einstellungen|Impostazioni indicizzatore|Definições do indexador
Import & Naming|Importation et nommage|Import und Benennung|Importazione e nomi|Importação e nomes
Import completed downloads|Importer les téléchargements terminés|Abgeschlossene Downloads importieren|Importa download completati|Importar transferências concluídas
Import method|Méthode d'importation|Importmethode|Metodo di importazione|Método de importação
Indexed Storage|Stockage indexé|Indexierter Speicher|Spazio indicizzato|Armazenamento indexado
Indexer|Indexeur|Indexer|Indicizzatore|Indexador
Indexer Activity|Activité des indexeurs|Indexer-Aktivität|Attività indicizzatori|Atividade dos indexadores
Indexer ID|ID de l'indexeur|Indexer-ID|ID indicizzatore|ID do indexador
Inherit:|Hériter :|Übernehmen:|Eredita:|Herdar:
Interval (hours)|Intervalle (heures)|Intervall (Stunden)|Intervallo (ore)|Intervalo (horas)
Interval (minutes)|Intervalle (minutes)|Intervall (Minuten)|Intervallo (minuti)|Intervalo (minutos)
Keep .torrent + mapping for future reseed|Conserver .torrent et association pour reseed|.torrent und Zuordnung für Reseed behalten|Conserva .torrent e associazione per reseed|Manter .torrent e associação para reseed
Keep latest copies|Sauvegardes récentes à conserver|Letzte Sicherungen behalten|Copie recenti da conservare|Cópias recentes a manter
Language|Langue|Sprache|Lingua|Idioma
Language Profile|Profil de langue|Sprachprofil|Profilo lingua|Perfil de idioma
Language Profiles|Profils de langue|Sprachprofile|Profili lingua|Perfis de idioma
Library|Bibliothèque|Bibliothek|Libreria|Biblioteca
Last 90 seconds|90 dernières secondes|Letzte 90 Sekunden|Ultimi 90 secondi|Últimos 90 segundos
Last Check|Dernière vérification|Letzte Prüfung|Ultimo controllo|Última verificação
Latency|Latence|Latenz|Latenza|Latência
Latest Season|Dernière saison|Letzte Staffel|Ultima stagione|Última temporada
Library Maintenance|Maintenance de la bibliothèque|Bibliothekswartung|Manutenzione libreria|Manutenção da biblioteca
Library Overview|Vue de la bibliothèque|Bibliotheksübersicht|Panoramica libreria|Resumo da biblioteca
Library entries|Éléments de bibliothèque|Bibliothekseinträge|Elementi della libreria|Elementos da biblioteca
Light theme|Thème clair|Helles Design|Tema chiaro|Tema claro
Loaded|Chargé|Geladen|Caricato|Carregado
Loading qBittorrent…|Chargement de qBittorrent…|qBittorrent wird geladen…|Caricamento qBittorrent…|A carregar qBittorrent…
Loading seasons…|Chargement des saisons…|Staffeln werden geladen…|Caricamento stagioni…|A carregar temporadas…
Loading settings…|Chargement des paramètres…|Einstellungen werden geladen…|Caricamento impostazioni…|A carregar definições…
Local path (as Oberiz sees it)|Chemin local (visible par Oberiz)|Lokaler Pfad (aus Sicht von Oberiz)|Percorso locale (visibile a Oberiz)|Caminho local (visível ao Oberiz)
Manual search|Recherche manuelle|Manuelle Suche|Ricerca manuale|Pesquisa manual
Magnet URL|URL magnet|Magnet-URL|URL magnet|URL magnet
Manage categories, save paths and tags|Gérer catégories, dossiers et étiquettes|Kategorien, Pfade und Tags verwalten|Gestisci categorie, percorsi ed etichette|Gerir categorias, caminhos e etiquetas
Manage your movie collection, downloads and monitoring|Gérez vos films, téléchargements et suivis|Filme, Downloads und Überwachung verwalten|Gestisci film, download e monitoraggio|Gerir filmes, transferências e monitorização
Manual|Manuel|Manuell|Manuale|Manual
Max episode size MB|Taille maximale d'épisode (Mo)|Maximale Episodengröße (MB)|Dimensione massima episodio (MB)|Tamanho máximo do episódio (MB)
Max season/pack size MB|Taille maximale de saison/pack (Mo)|Maximale Staffel-/Paketgröße (MB)|Dimensione massima stagione/pacchetto (MB)|Tamanho máximo da temporada/pacote (MB)
Media|Contenu|Medien|Contenuti|Conteúdo
Media type|Type de contenu|Medientyp|Tipo di contenuto|Tipo de conteúdo
Metadata, posters and media information|Métadonnées, affiches et informations|Metadaten, Poster und Medieninformationen|Metadati, locandine e informazioni|Metadados, cartazes e informações
Method|Méthode|Methode|Metodo|Método
Middle theme|Thème intermédiaire|Mittleres Design|Tema intermedio|Tema intermédio
Min seeds|Sources minimales|Minimale Seeds|Seed minimi|Sementes mínimas
Min size MB|Taille minimale (Mo)|Mindestgröße (MB)|Dimensione minima (MB)|Tamanho mínimo (MB)
Minimum ratio|Ratio minimal|Mindestverhältnis|Rapporto minimo|Rácio mínimo
Minimum seed days|Jours de partage minimaux|Mindest-Seeding-Tage|Giorni minimi di condivisione|Dias mínimos de partilha
Minimum seeds|Sources minimales|Minimale Seeds|Seed minimi|Sementes mínimas
Missing Episodes|Épisodes manquants|Fehlende Episoden|Episodi mancanti|Episódios em falta
Missing · quality upgrades|Manquants · améliorations|Fehlend · Qualitätsverbesserungen|Mancanti · miglioramenti di qualità|Em falta · melhorias de qualidade
Monitor|Suivre|Überwachen|Monitora|Monitorizar
Monitor Mode|Mode de suivi|Überwachungsmodus|Modalità di monitoraggio|Modo de monitorização
Monitored|Suivi|Überwacht|Monitorato|Monitorizado
Monitored Movies|Films suivis|Überwachte Filme|Film monitorati|Filmes monitorizados
Monitored only|Suivis uniquement|Nur überwachte|Solo monitorati|Só monitorizados
More|Plus|Mehr|Altro|Mais
More series options|Plus d'options de séries|Weitere Serienoptionen|Altre opzioni serie|Mais opções de séries
Move (only for setups without seeding)|Déplacer (sans partage uniquement)|Verschieben (nur ohne Seeding)|Sposta (solo senza condivisione)|Mover (só sem partilha)
Movie|Film|Film|Film|Filme
Movie Profiles|Profils de films|Filmprofile|Profili film|Perfis de filmes
Movie naming template|Modèle de nom des films|Benennungsvorlage für Filme|Modello nomi film|Modelo de nomes de filmes
Movies Library|Bibliothèque de films|Filmbibliothek|Libreria film|Biblioteca de filmes
Movies and series metadata|Métadonnées des films et séries|Film- und Serienmetadaten|Metadati di film e serie|Metadados de filmes e séries
New download|Nouveau téléchargement|Neuer Download|Nuovo download|Nova transferência
Name|Nom|Name|Nome|Nome
Needs Test|À tester|Test erforderlich|Da verificare|Requer teste
Never auto-delete|Ne jamais supprimer automatiquement|Nie automatisch löschen|Mai eliminare automaticamente|Nunca eliminar automaticamente
New Language Profile|Nouveau profil de langue|Neues Sprachprofil|Nuovo profilo lingua|Novo perfil de idioma
New Profile|Nouveau profil|Neues Profil|Nuovo profilo|Novo perfil
New password|Nouveau mot de passe|Neues Passwort|Nuova password|Nova palavra-passe
New tag|Nouvelle étiquette|Neues Tag|Nuova etichetta|Nova etiqueta
No profile|Aucun profil|Kein Profil|Nessun profilo|Sem perfil
No recent activity|Aucune activité récente|Keine aktuelle Aktivität|Nessuna attività recente|Sem atividade recente
No subfolders.|Aucun sous-dossier.|Keine Unterordner.|Nessuna sottocartella.|Sem subpastas.
No upcoming episodes|Aucun épisode à venir|Keine kommenden Episoden|Nessun episodio in arrivo|Sem próximos episódios
None|Aucun|Keine|Nessuno|Nenhum
Not Monitored|Non suivi|Nicht überwacht|Non monitorato|Não monitorizado
On Hold|En attente|Zurückgestellt|In sospeso|Em espera
Online|En ligne|Online|Online|Online
Open calendar|Ouvrir le calendrier|Kalender öffnen|Apri calendario|Abrir calendário
Open history|Ouvrir l'historique|Verlauf öffnen|Apri cronologia|Abrir histórico
Open imports|Ouvrir les importations|Importe öffnen|Apri importazioni|Abrir importações
Open menu|Ouvrir le menu|Menü öffnen|Apri menu|Abrir menu
Open notifications|Ouvrir les notifications|Benachrichtigungen öffnen|Apri notifiche|Abrir notificações
Open seasons|Ouvrir les saisons|Staffeln öffnen|Apri stagioni|Abrir temporadas
Open settings|Ouvrir les paramètres|Einstellungen öffnen|Apri impostazioni|Abrir definições
Open system diagnostics|Ouvrir le diagnostic|Systemdiagnose öffnen|Apri diagnostica|Abrir diagnóstico
Password|Mot de passe|Passwort|Password|Palavra-passe
Paths|Chemins|Pfade|Percorsi|Caminhos
PayPal donation QR code for Oberiz|QR code de don PayPal pour Oberiz|PayPal-Spenden-QR-Code für Oberiz|Codice QR donazione PayPal per Oberiz|Código QR de donativo PayPal para Oberiz
Peers|Pairs|Peers|Peer|Pares
Port|Port|Port|Porta|Porta
Prefer season/complete packs|Préférer les packs de saisons/séries|Staffel-/Komplettpakete bevorzugen|Preferisci pacchetti stagioni/serie|Preferir pacotes de temporadas/séries
Preferred terms (TERM:score)|Termes préférés (TERME:score)|Bevorzugte Begriffe (BEGRIFF:Punkte)|Termini preferiti (TERMINE:punteggio)|Termos preferidos (TERMO:pontuação)
Prioritize indexer|Prioriser l'indexeur|Indexer priorisieren|Dai priorità all'indicizzatore|Priorizar indexador
Priority|Priorité|Priorität|Priorità|Prioridade
Profile|Profil|Profil|Profilo|Perfil
Profile:|Profil :|Profil:|Profilo:|Perfil:
Progress|Progression|Fortschritt|Avanzamento|Progresso
Prowlarr upstream|Catalogue Prowlarr|Prowlarr-Katalog|Catalogo Prowlarr|Catálogo Prowlarr
Public API v1|API publique v1|Öffentliche API v1|API pubblica v1|API pública v1
Quality Profile|Profil de qualité|Qualitätsprofil|Profilo qualità|Perfil de qualidade
Quality Profiles|Profils de qualité|Qualitätsprofile|Profili qualità|Perfis de qualidade
Queue Health|État de la file|Warteschlangenstatus|Stato della coda|Estado da fila
RSS Automation|Automatisation RSS|RSS-Automatisierung|Automazione RSS|Automatização RSS
RSS feed URL|URL du flux RSS|RSS-Feed-URL|URL del feed RSS|URL do feed RSS
Recent Activity|Activité récente|Letzte Aktivität|Attività recente|Atividade recente
Recent Completed|Terminés récemment|Kürzlich abgeschlossen|Completati di recente|Concluídos recentemente
Recent Downloads|Téléchargements récents|Letzte Downloads|Download recenti|Transferências recentes
Recent errors:|Erreurs récentes :|Letzte Fehler:|Errori recenti:|Erros recentes:
Recently Added|Ajoutés récemment|Kürzlich hinzugefügt|Aggiunti di recente|Adicionados recentemente
Recently Added / Wanted|Ajoutés récemment / Recherchés|Kürzlich hinzugefügt / Gesucht|Aggiunti di recente / Richiesti|Adicionados recentemente / Procurados
Refresh|Actualiser|Aktualisieren|Aggiorna|Atualizar
Refresh diagnostics|Actualiser le diagnostic|Diagnose aktualisieren|Aggiorna diagnostica|Atualizar diagnóstico
Reject|Rejeter|Ablehnen|Rifiuta|Rejeitar
Reject release|Rejeter cette version|Veröffentlichung ablehnen|Rifiuta versione|Rejeitar versão
Reject terms|Termes rejetés|Abgelehnte Begriffe|Termini rifiutati|Termos rejeitados
Rejected releases (|Versions rejetées (|Abgelehnte Veröffentlichungen (|Versioni rifiutate (|Versões rejeitadas (
Reload|Recharger|Neu laden|Ricarica|Recarregar
Remote path (as qBittorrent sees it)|Chemin distant (visible par qBittorrent)|Entfernter Pfad (aus Sicht von qBittorrent)|Percorso remoto (visibile a qBittorrent)|Caminho remoto (visível ao qBittorrent)
Remove|Supprimer|Entfernen|Rimuovi|Remover
Remove from configured indexers|Supprimer des indexeurs configurés|Aus konfigurierten Indexern entfernen|Rimuovi dagli indicizzatori configurati|Remover dos indexadores configurados
Remove password|Supprimer le mot de passe|Passwort entfernen|Rimuovi password|Remover palavra-passe
Remove torrent + original download|Supprimer torrent et téléchargement original|Torrent und Originaldownload entfernen|Rimuovi torrent e download originale|Remover torrent e transferência original
Remove torrent, keep original download|Supprimer le torrent, conserver le téléchargement|Torrent entfernen, Originaldownload behalten|Rimuovi torrent, conserva download originale|Remover torrent, manter transferência original
Rename imported files|Renommer les fichiers importés|Importierte Dateien umbenennen|Rinomina file importati|Mudar nomes dos ficheiros importados
Requirement|Condition|Anforderung|Requisito|Requisito
Rescan All|Tout réanalyser|Alles neu einlesen|Scansiona tutto|Analisar tudo novamente
Rescan Movies|Réanalyser les films|Filme neu einlesen|Scansiona film|Analisar filmes novamente
Rescan Series|Réanalyser les séries|Serien neu einlesen|Scansiona serie|Analisar séries novamente
Reseed workspace|Dossier de reseed|Reseed-Arbeitsverzeichnis|Cartella di reseed|Pasta de reseed
Restore|Restaurer|Wiederherstellen|Ripristina|Restaurar
Save|Enregistrer|Speichern|Salva|Guardar
Save Language Profile|Enregistrer le profil de langue|Sprachprofil speichern|Salva profilo lingua|Guardar perfil de idioma
Save Policy|Enregistrer la règle|Richtlinie speichern|Salva regola|Guardar política
Save Profile|Enregistrer le profil|Profil speichern|Salva profilo|Guardar perfil
Save Settings|Enregistrer les paramètres|Einstellungen speichern|Salva impostazioni|Guardar definições
Save path (optional)|Dossier d'enregistrement (facultatif)|Speicherpfad (optional)|Percorso di salvataggio (facoltativo)|Caminho de gravação (opcional)
Schedule automatic backups|Planifier des sauvegardes automatiques|Automatische Sicherungen planen|Pianifica backup automatici|Agendar cópias de segurança automáticas
Score|Score|Punktzahl|Punteggio|Pontuação
Search|Rechercher|Suchen|Cerca|Pesquisar
Search Pack|Rechercher un pack|Paket suchen|Cerca pacchetto|Pesquisar pacote
Search Prowlarr and custom definitions...|Rechercher des définitions Prowlarr et personnalisées…|Prowlarr- und eigene Definitionen suchen…|Cerca definizioni Prowlarr e personalizzate…|Pesquisar definições Prowlarr e personalizadas…
Search configured indexers...|Rechercher les indexeurs configurés…|Konfigurierte Indexer suchen…|Cerca indicizzatori configurati…|Pesquisar indexadores configurados…
Search downloads|Rechercher des téléchargements|Downloads suchen|Cerca download|Pesquisar transferências
Search downloads...|Rechercher des téléchargements…|Downloads suchen…|Cerca download…|Pesquisar transferências…
Search movies...|Rechercher des films…|Filme suchen…|Cerca film…|Pesquisar filmes…
Search releases|Rechercher des versions|Veröffentlichungen suchen|Cerca versioni|Pesquisar versões
Search series...|Rechercher des séries…|Serien suchen…|Cerca serie…|Pesquisar séries…
Searching enabled indexers…|Recherche dans les indexeurs actifs…|Aktive Indexer werden durchsucht…|Ricerca negli indicizzatori attivi…|A pesquisar nos indexadores ativos…
Seed Policies|Règles de partage|Seeding-Richtlinien|Regole di condivisione|Políticas de partilha
Seeding|Partage|Wird geteilt|Condivisione|A partilhar
Seeds|Sources|Seeds|Seed|Sementes
Seeds / Peers|Sources / Pairs|Seeds / Peers|Seed / Peer|Sementes / Pares
Select|Sélectionner|Auswählen|Seleziona|Selecionar
Select folder|Choisir un dossier|Ordner auswählen|Seleziona cartella|Selecionar pasta
Self-hosted Media Automation|Automatisation multimédia auto-hébergée|Selbst gehostete Medienautomatisierung|Automazione multimediale autonoma|Automatização multimédia autoalojada
Series Library|Bibliothèque de séries|Serienbibliothek|Libreria serie|Biblioteca de séries
Series Profiles|Profils de séries|Serienprofile|Profili serie|Perfis de séries
Series identifiers for episode matching|Identifiants de séries pour associer les épisodes|Serienkennungen zur Episodenzuordnung|Identificatori serie per associare episodi|Identificadores de séries para associar episódios
Series naming template|Modèle de nom des séries|Benennungsvorlage für Serien|Modello nomi serie|Modelo de nomes de séries
Series release search|Recherche de versions de séries|Suche nach Serienveröffentlichungen|Ricerca versioni serie|Pesquisa de versões de séries
Set Default|Définir par défaut|Als Standard festlegen|Imposta predefinito|Definir como predefinido
Set an optional|Définissez un élément facultatif|Optional festlegen|Imposta un valore facoltativo|Defina um valor opcional
Settings → Services|Paramètres → Services|Einstellungen → Dienste|Impostazioni → Servizi|Definições → Serviços
Show rejected releases|Afficher les versions rejetées|Abgelehnte Veröffentlichungen anzeigen|Mostra versioni rifiutate|Mostrar versões rejeitadas
Sign in|Se connecter|Anmelden|Accedi|Iniciar sessão
Sign out|Se déconnecter|Abmelden|Esci|Terminar sessão
Size|Taille|Größe|Dimensione|Tamanho
Sort|Trier|Sortieren|Ordina|Ordenar
Sort by: Date Added|Trier par : date d'ajout|Sortieren nach: Hinzufügedatum|Ordina per: data aggiunta|Ordenar por: data de adição
Sort by: Recently Added|Trier par : ajouts récents|Sortieren nach: zuletzt hinzugefügt|Ordina per: aggiunti di recente|Ordenar por: adicionados recentemente
Sort by: Title|Trier par : titre|Sortieren nach: Titel|Ordina per: titolo|Ordenar por: título
Sort by: Year|Trier par : année|Sortieren nach: Jahr|Ordina per: anno|Ordenar por: ano
Sort downloads by|Trier les téléchargements par|Downloads sortieren nach|Ordina download per|Ordenar transferências por
Speed|Vitesse|Geschwindigkeit|Velocità|Velocidade
Stalled|Sans activité|Ohne Aktivität|Inattivo|Sem atividade
Status|État|Status|Stato|Estado
Support Oberiz|Soutenir Oberiz|Oberiz unterstützen|Sostieni Oberiz|Apoiar o Oberiz
Support via PayPal|Soutenir via PayPal|Über PayPal unterstützen|Sostieni tramite PayPal|Apoiar via PayPal
Sync Now|Synchroniser maintenant|Jetzt synchronisieren|Sincronizza ora|Sincronizar agora
System Status|État du système|Systemstatus|Stato del sistema|Estado do sistema
Tags|Étiquettes|Tags|Etichette|Etiquetas
Test|Tester|Testen|Verifica|Testar
Test Connection|Tester la connexion|Verbindung testen|Verifica connessione|Testar ligação
Time|Heure|Zeit|Ora|Hora
Title|Titre|Titel|Titolo|Título
Torrent metadata|Métadonnées du torrent|Torrent-Metadaten|Metadati torrent|Metadados do torrent
Total Movies|Total de films|Filme insgesamt|Totale film|Total de filmes
Total Series|Total de séries|Serien insgesamt|Totale serie|Total de séries
Total Speed|Vitesse totale|Gesamtgeschwindigkeit|Velocità totale|Velocidade total
Tracker / source|Tracker / source|Tracker / Quelle|Tracker / sorgente|Tracker / fonte
Type|Type|Typ|Tipo|Tipo
Unavailable|Indisponible|Nicht verfügbar|Non disponibile|Indisponível
Uncategorized|Sans catégorie|Ohne Kategorie|Senza categoria|Sem categoria
Uncategorized / Sin categorizar|Sans catégorie|Ohne Kategorie|Senza categoria|Sem categoria
Undo|Annuler|Rückgängig|Annulla|Desfazer
Undo reject|Annuler le rejet|Ablehnung aufheben|Annulla rifiuto|Desfazer rejeição
Unknown language:|Langue inconnue :|Unbekannte Sprache:|Lingua sconosciuta:|Idioma desconhecido:
Upcoming|À venir|Demnächst|In arrivo|Próximos
Upcoming Episodes|Épisodes à venir|Kommende Episoden|Episodi in arrivo|Próximos episódios
Upcoming missing|À venir et manquants|Kommend und fehlend|In arrivo e mancanti|Próximos em falta
Upgrade cutoff score|Score limite d'amélioration|Grenzwert für Verbesserungen|Punteggio limite miglioramenti|Pontuação limite de melhorias
Upgrade wanted|Amélioration recherchée|Verbesserung gesucht|Miglioramento richiesto|Melhoria procurada
Upgrades wanted|Améliorations recherchées|Verbesserungen gesucht|Miglioramenti richiesti|Melhorias procuradas
Upload backup|Téléverser une sauvegarde|Sicherung hochladen|Carica backup|Enviar cópia de segurança
Upload speed|Vitesse d'envoi|Upload-Geschwindigkeit|Velocità di upload|Velocidade de envio
Upstream indexers|Indexeurs du catalogue|Katalog-Indexer|Indicizzatori del catalogo|Indexadores do catálogo
Use HTTPS|Utiliser HTTPS|HTTPS verwenden|Usa HTTPS|Usar HTTPS
Use this folder|Utiliser ce dossier|Diesen Ordner verwenden|Usa questa cartella|Usar esta pasta
Username|Nom d'utilisateur|Benutzername|Nome utente|Nome de utilizador
View All|Tout voir|Alle anzeigen|Mostra tutto|Ver tudo
View Calendar|Voir le calendrier|Kalender anzeigen|Mostra calendario|Ver calendário
Wanted|Recherché|Gesucht|Richiesto|Procurado
Wanted / Upgrades|Recherchés / Améliorations|Gesucht / Verbesserungen|Richiesti / Miglioramenti|Procurados / Melhorias
Your configured indexers|Vos indexeurs configurés|Ihre konfigurierten Indexer|I tuoi indicizzatori configurati|Os seus indexadores configurados
Your download queue is clear|Votre file de téléchargement est vide|Die Download-Warteschlange ist leer|La coda di download è vuota|A fila de transferências está vazia
Accepted|Accepté|Akzeptiert|Accettato|Aceite
definitions available to configure|définitions disponibles à configurer|Definitionen zur Konfiguration verfügbar|definizioni disponibili da configurare|definições disponíveis para configurar
enabled not tested yet|activés mais non testés|aktiviert, noch nicht getestet|attivi ma non verificati|ativos ainda não testados
enabled online|activés en ligne|aktiviert und online|attivi online|ativos online
enabled · Automation:|activés · Automatisation :|aktiviert · Automatisierung:|attivi · Automazione:|ativos · Automatização:
episodes|épisodes|Episoden|episodi|episódios
Files|Fichiers|Dateien|File|Ficheiros
Grabbed|Envoyé|Übergeben|Inviato|Enviado
indexer errors|erreurs d'indexeur|Indexer-Fehler|errori indicizzatori|erros de indexadores
Matched|Associé|Zugeordnet|Associato|Associado
obsolete record|enregistrement obsolète|veralteter Eintrag|record obsoleto|registo obsoleto
of|sur|von|di|de
qBittorrent Organization|Organisation qBittorrent|qBittorrent-Organisation|Organizzazione qBittorrent|Organização qBittorrent
qBittorrent Tags|Étiquettes qBittorrent|qBittorrent-Tags|Etichette qBittorrent|Etiquetas qBittorrent
qBittorrent connection error|Erreur de connexion qBittorrent|qBittorrent-Verbindungsfehler|Errore connessione qBittorrent|Erro de ligação qBittorrent
qBittorrent unavailable:|qBittorrent indisponible :|qBittorrent nicht verfügbar:|qBittorrent non disponibile:|qBittorrent indisponível:
Season|Saison|Staffel|Stagione|Temporada
Shown|Affichés|Angezeigt|Mostrati|Mostrados
total in library|total en bibliothèque|insgesamt in der Bibliothek|totale in libreria|total na biblioteca
· Database:|· Base de données :|· Datenbank:|· Database:|· Base de dados:
· Indexers:|· Indexeurs :|· Indexer:|· Indicizzatori:|· Indexadores:
← Back|← Retour|← Zurück|← Indietro|← Voltar
↑ Parent folder|↑ Dossier parent|↑ Übergeordneter Ordner|↑ Cartella superiore|↑ Pasta superior
↻ Refresh|↻ Actualiser|↻ Aktualisieren|↻ Aggiorna|↻ Atualizar
API key copied to clipboard.|Clé API copiée.|API-Schlüssel kopiert.|Chiave API copiata.|Chave API copiada.
API key or Read Access Token|Clé API ou jeton de lecture|API-Schlüssel oder Lesezugriffstoken|Chiave API o token di lettura|Chave API ou token de leitura
Action failed|Échec de l'action|Aktion fehlgeschlagen|Azione non riuscita|A ação falhou
Active records cannot be removed|Les enregistrements actifs ne peuvent pas être supprimés|Aktive Einträge können nicht entfernt werden|I record attivi non possono essere rimossi|Não é possível remover registos ativos
Adding…|Ajout…|Wird hinzugefügt…|Aggiunta…|A adicionar…
Admin password removed.|Mot de passe administrateur supprimé.|Administratorpasswort entfernt.|Password amministratore rimossa.|Palavra-passe de administrador removida.
Ascending|Croissant|Aufsteigend|Crescente|Ascendente
Ascending sort|Tri croissant|Aufsteigend sortieren|Ordine crescente|Ordem ascendente
Automation checked media|Contenu vérifié par l'automatisation|Von Automatisierung geprüfte Medien|Contenuti verificati dall'automazione|Conteúdo verificado pela automatização
Automation needs attention|L'automatisation nécessite une intervention|Automatisierung erfordert Aufmerksamkeit|L'automazione richiede attenzione|A automatização requer atenção
Browse folders|Parcourir les dossiers|Ordner durchsuchen|Sfoglia cartelle|Procurar pastas
Change password|Changer le mot de passe|Passwort ändern|Cambia password|Alterar palavra-passe
Check Settings connection|Vérifier la connexion dans les paramètres|Verbindung in Einstellungen prüfen|Verifica connessione nelle impostazioni|Verificar ligação nas definições
Check download queue|Vérifier la file de téléchargement|Download-Warteschlange prüfen|Verifica coda download|Verificar fila de transferências
Checking|Vérification|Wird geprüft|Verifica in corso|A verificar
Clone profile|Cloner le profil|Profil klonen|Clona profilo|Clonar perfil
Collapse menu|Réduire le menu|Menü einklappen|Comprimi menu|Recolher menu
Collecting diagnostic data…|Collecte des données de diagnostic…|Diagnosedaten werden gesammelt…|Raccolta dati diagnostici…|A recolher dados de diagnóstico…
Connected|Connecté|Verbunden|Connesso|Ligado
Continue|Continuer|Weiter|Continua|Continuar
Copied|Copié|Kopiert|Copiato|Copiado
Could not add magnet|Impossible d'ajouter le magnet|Magnet konnte nicht hinzugefügt werden|Impossibile aggiungere magnet|Não foi possível adicionar o magnet
Could not create category|Impossible de créer la catégorie|Kategorie konnte nicht erstellt werden|Impossibile creare categoria|Não foi possível criar a categoria
Could not create tag|Impossible de créer l'étiquette|Tag konnte nicht erstellt werden|Impossibile creare etichetta|Não foi possível criar a etiqueta
Could not delete torrent|Impossible de supprimer le torrent|Torrent konnte nicht gelöscht werden|Impossibile eliminare torrent|Não foi possível eliminar o torrent
Could not load downloads|Impossible de charger les téléchargements|Downloads konnten nicht geladen werden|Impossibile caricare download|Não foi possível carregar as transferências
Could not load indexers|Impossible de charger les indexeurs|Indexer konnten nicht geladen werden|Impossibile caricare indicizzatori|Não foi possível carregar os indexadores
Could not load qBittorrent organization|Impossible de charger l'organisation qBittorrent|qBittorrent-Organisation konnte nicht geladen werden|Impossibile caricare organizzazione qBittorrent|Não foi possível carregar a organização qBittorrent
Could not reach qBittorrent|Impossible de joindre qBittorrent|qBittorrent nicht erreichbar|Impossibile connettersi a qBittorrent|Não foi possível ligar ao qBittorrent
Could not reject release|Impossible de rejeter cette version|Veröffentlichung konnte nicht abgelehnt werden|Impossibile rifiutare versione|Não foi possível rejeitar a versão
Could not remove category|Impossible de supprimer la catégorie|Kategorie konnte nicht entfernt werden|Impossibile rimuovere categoria|Não foi possível remover a categoria
Could not remove tag|Impossible de supprimer l'étiquette|Tag konnte nicht entfernt werden|Impossibile rimuovere etichetta|Não foi possível remover a etiqueta
Could not undo the rejection|Impossible d'annuler le rejet|Ablehnung konnte nicht aufgehoben werden|Impossibile annullare il rifiuto|Não foi possível desfazer a rejeição
Could not update category|Impossible de modifier la catégorie|Kategorie konnte nicht aktualisiert werden|Impossibile aggiornare categoria|Não foi possível atualizar a categoria
Could not update torrent organization|Impossible de modifier l'organisation du torrent|Torrent-Organisation konnte nicht aktualisiert werden|Impossibile aggiornare organizzazione torrent|Não foi possível atualizar a organização do torrent
Creating backup…|Création de la sauvegarde…|Sicherung wird erstellt…|Creazione backup…|A criar cópia de segurança…
Current|Actuel|Aktuell|Attuale|Atual
Delete torrent|Supprimer le torrent|Torrent löschen|Elimina torrent|Eliminar torrent
Delete torrent + files|Supprimer torrent et fichiers|Torrent und Dateien löschen|Elimina torrent e file|Eliminar torrent e ficheiros
Deleting backup…|Suppression de la sauvegarde…|Sicherung wird gelöscht…|Eliminazione backup…|A eliminar cópia de segurança…
Descending|Décroissant|Absteigend|Decrescente|Descendente
Descending sort|Tri décroissant|Absteigend sortieren|Ordine decrescente|Ordem descendente
Disable|Désactiver|Deaktivieren|Disattiva|Desativar
Edit language profile|Modifier le profil de langue|Sprachprofil bearbeiten|Modifica profilo lingua|Editar perfil de idioma
Edit profile|Modifier le profil|Profil bearbeiten|Modifica profilo|Editar perfil
Enable|Activer|Aktivieren|Attiva|Ativar
Everything looks good|Tout fonctionne correctement|Alles funktioniert|Tutto funziona correttamente|Tudo funciona corretamente
Expand menu|Développer le menu|Menü ausklappen|Espandi menu|Expandir menu
Expand to load episode metadata|Développer pour charger les métadonnées|Ausklappen, um Episodenmetadaten zu laden|Espandi per caricare metadati episodi|Expandir para carregar metadados dos episódios
Finished|Terminé|Beendet|Terminato|Terminado
For example: ThePirateBay|Par exemple : ThePirateBay|Zum Beispiel: ThePirateBay|Ad esempio: ThePirateBay|Por exemplo: ThePirateBay
Games|Jeux|Spiele|Giochi|Jogos
Generate key|Générer une clé|Schlüssel erzeugen|Genera chiave|Gerar chave
Generate or enter a key|Générez ou saisissez une clé|Schlüssel erzeugen oder eingeben|Genera o inserisci una chiave|Gere ou introduza uma chave
Grab|Télécharger|Herunterladen|Scarica|Transferir
Hide|Masquer|Ausblenden|Nascondi|Ocultar
Idle|Inactif|Inaktiv|Inattivo|Inativo
Import needs attention|L'importation nécessite une intervention|Import erfordert Aufmerksamkeit|L'importazione richiede attenzione|A importação requer atenção
Imported to library|Importé dans la bibliothèque|In Bibliothek importiert|Importato nella libreria|Importado para a biblioteca
Indexer ID is required|L'ID de l'indexeur est requis|Indexer-ID ist erforderlich|L'ID indicizzatore è obbligatorio|O ID do indexador é obrigatório
Keep current if blank|Vide pour conserver l'actuel|Leer lassen, um aktuellen Wert zu behalten|Vuoto per mantenere il valore attuale|Vazio para manter o atual
Keep this series under automation.|Maintenir cette série dans l'automatisation.|Diese Serie weiter automatisieren.|Mantieni questa serie nell'automazione.|Manter esta série na automatização.
Leave blank to keep current|Laisser vide pour conserver l'actuel|Leer lassen, um aktuellen Wert zu behalten|Lascia vuoto per mantenere il valore attuale|Deixar vazio para manter o atual
Loading…|Chargement…|Wird geladen…|Caricamento…|A carregar…
Lower number is preferred|Le nombre le plus bas est préféré|Kleinere Zahl wird bevorzugt|È preferito il numero più basso|É preferido o número menor
Metadata|Métadonnées|Metadaten|Metadati|Metadados
Moving|Déplacement|Wird verschoben|Spostamento|A mover
Never|Jamais|Nie|Mai|Nunca
New Movie Profile|Nouveau profil de films|Neues Filmprofil|Nuovo profilo film|Novo perfil de filmes
New Series Profile|Nouveau profil de séries|Neues Serienprofil|Nuovo profilo serie|Novo perfil de séries
Next episode|Prochain épisode|Nächste Episode|Prossimo episodio|Próximo episódio
No active transfers|Aucun transfert actif|Keine aktiven Übertragungen|Nessun trasferimento attivo|Sem transferências ativas
No air date|Pas de date de diffusion|Kein Ausstrahlungsdatum|Nessuna data di trasmissione|Sem data de emissão
No description available.|Aucune description disponible.|Keine Beschreibung verfügbar.|Nessuna descrizione disponibile.|Sem descrição disponível.
No feeds configured|Aucun flux configuré|Keine Feeds konfiguriert|Nessun feed configurato|Sem feeds configurados
No matching downloads|Aucun téléchargement correspondant|Keine passenden Downloads|Nessun download corrispondente|Sem transferências correspondentes
No overview|Pas de synopsis|Keine Zusammenfassung|Nessuna trama|Sem sinopse
No overview available.|Aucun synopsis disponible.|Keine Zusammenfassung verfügbar.|Nessuna trama disponibile.|Sem sinopse disponível.
No releases found in the enabled indexers.|Aucune version trouvée dans les indexeurs actifs.|Keine Veröffentlichungen in aktiven Indexern gefunden.|Nessuna versione trovata negli indicizzatori attivi.|Não foram encontradas versões nos indexadores ativos.
No releases pass the selected profile.|Aucune version ne respecte le profil.|Keine Veröffentlichung erfüllt das gewählte Profil.|Nessuna versione soddisfa il profilo scelto.|Nenhuma versão cumpre o perfil selecionado.
No tags|Aucune étiquette|Keine Tags|Nessuna etichetta|Sem etiquetas
No torrents yet|Aucun torrent pour le moment|Noch keine Torrents|Nessun torrent ancora|Ainda sem torrents
Not configured|Non configuré|Nicht konfiguriert|Non configurato|Não configurado
Not tested|Non testé|Nicht getestet|Non verificato|Não testado
OFF|DÉSACTIVÉ|AUS|DISATTIVATO|DESATIVADO
ON|ACTIVÉ|EIN|ATTIVATO|ATIVADO
Password must be at least 8 characters|Le mot de passe doit comporter au moins 8 caractères|Das Passwort muss mindestens 8 Zeichen lang sein|La password deve avere almeno 8 caratteri|A palavra-passe deve ter pelo menos 8 caracteres
Passwords do not match|Les mots de passe ne correspondent pas|Die Passwörter stimmen nicht überein|Le password non coincidono|As palavras-passe não coincidem
Preparing next search…|Préparation de la recherche suivante…|Nächste Suche wird vorbereitet…|Preparazione prossima ricerca…|A preparar a próxima pesquisa…
Private|Privé|Privat|Privato|Privado
Public|Public|Öffentlich|Pubblico|Público
Queue needs attention|La file nécessite une intervention|Warteschlange erfordert Aufmerksamkeit|La coda richiede attenzione|A fila requer atenção
RSS synchronization completed|Synchronisation RSS terminée|RSS-Synchronisierung abgeschlossen|Sincronizzazione RSS completata|Sincronização RSS concluída
RSS synchronization failed|Échec de synchronisation RSS|RSS-Synchronisierung fehlgeschlagen|Sincronizzazione RSS non riuscita|A sincronização RSS falhou
Refresh TMDB|Actualiser TMDB|TMDB aktualisieren|Aggiorna TMDB|Atualizar TMDB
Refreshing…|Actualisation…|Wird aktualisiert…|Aggiornamento…|A atualizar…
Regenerate key|Régénérer la clé|Schlüssel neu erzeugen|Rigenera chiave|Gerar nova chave
Reject + delete files|Rejeter et supprimer les fichiers|Ablehnen und Dateien löschen|Rifiuta ed elimina file|Rejeitar e eliminar ficheiros
Reject this release?|Rejeter cette version ?|Diese Veröffentlichung ablehnen?|Rifiutare questa versione?|Rejeitar esta versão?
Reload Definitions|Recharger les définitions|Definitionen neu laden|Ricarica definizioni|Recarregar definições
Remove download?|Supprimer le téléchargement ?|Download entfernen?|Rimuovere il download?|Remover transferência?
Removing…|Suppression…|Wird entfernt…|Rimozione…|A remover…
Restoring backup…|Restauration de la sauvegarde…|Sicherung wird wiederhergestellt…|Ripristino backup…|A restaurar cópia de segurança…
Run Now|Exécuter maintenant|Jetzt ausführen|Esegui ora|Executar agora
Running automation and checking qBittorrent…|Automatisation et vérification de qBittorrent…|Automatisierung läuft und prüft qBittorrent…|Automazione e verifica qBittorrent…|A executar automatização e verificar qBittorrent…
Running…|Exécution…|Wird ausgeführt…|In esecuzione…|Em execução…
Saved|Enregistré|Gespeichert|Salvato|Guardado
Saving…|Enregistrement…|Wird gespeichert…|Salvataggio…|A guardar…
Scanning library…|Analyse de la bibliothèque…|Bibliothek wird eingelesen…|Scansione libreria…|A analisar biblioteca…
Search TMDB for a movie or series.|Recherchez un film ou une série sur TMDB.|Film oder Serie auf TMDB suchen.|Cerca film o serie su TMDB.|Pesquise um filme ou série no TMDB.
Search missing releases and allowed upgrades.|Rechercher les versions manquantes et améliorations autorisées.|Fehlende Veröffentlichungen und erlaubte Verbesserungen suchen.|Cerca versioni mancanti e miglioramenti consentiti.|Pesquisar versões em falta e melhorias permitidas.
Searching…|Recherche…|Wird gesucht…|Ricerca…|A pesquisar…
Sending…|Envoi…|Wird gesendet…|Invio…|A enviar…
Sent to qBittorrent|Envoyé à qBittorrent|An qBittorrent übergeben|Inviato a qBittorrent|Enviado ao qBittorrent
Set password|Définir le mot de passe|Passwort festlegen|Imposta password|Definir palavra-passe
Set — generate a new key to replace it|Définie ; générez une nouvelle clé pour la remplacer|Festgelegt; zum Ersetzen neuen Schlüssel erzeugen|Impostata; genera una nuova chiave per sostituirla|Definida; gere outra chave para substituir
Show|Afficher|Anzeigen|Mostra|Mostrar
Signing in…|Connexion…|Anmeldung…|Accesso…|A iniciar sessão…
Specials|Épisodes spéciaux|Specials|Speciali|Especiais
Start|Démarrer|Starten|Avvia|Iniciar
Starting…|Démarrage…|Wird gestartet…|Avvio…|A iniciar…
Stop|Arrêter|Stoppen|Ferma|Parar
Stopped|Arrêté|Gestoppt|Fermato|Parado
Syncing…|Synchronisation…|Synchronisierung…|Sincronizzazione…|A sincronizar…
Testing…|Test…|Wird getestet…|Verifica…|A testar…
Today|Aujourd'hui|Heute|Oggi|Hoje
Torrent removed from qBittorrent|Torrent supprimé de qBittorrent|Torrent aus qBittorrent entfernt|Torrent rimosso da qBittorrent|Torrent removido do qBittorrent
Try another filter or search term.|Essayez un autre filtre ou terme.|Anderen Filter oder Suchbegriff versuchen.|Prova un altro filtro o termine.|Experimente outro filtro ou termo.
Unknown|Inconnu|Unbekannt|Sconosciuto|Desconhecido
Unmonitor|Arrêter le suivi|Überwachung beenden|Non monitorare|Deixar de monitorizar
Unnamed torrent|Torrent sans nom|Unbenannter Torrent|Torrent senza nome|Torrent sem nome
Up to date|À jour|Aktuell|Aggiornato|Atualizado
Updated|Mis à jour|Aktualisiert|Aggiornato|Atualizado
Uploading backup…|Téléversement de la sauvegarde…|Sicherung wird hochgeladen…|Caricamento backup…|A enviar cópia de segurança…
Waiting…|En attente…|Warten…|In attesa…|A aguardar…
just now|à l'instant|gerade eben|adesso|agora mesmo
needs review|à vérifier|Prüfung erforderlich|da controllare|requer revisão
status unavailable|état indisponible|Status nicht verfügbar|stato non disponibile|estado indisponível
 · currently configured| · déjà configuré| · derzeit konfiguriert| · già configurato| · já configurado
`.trim().split('\n').map(line => line.split('|'))

export type EuropeanLanguage = 'fr' | 'de' | 'it' | 'pt'
export const european: Record<EuropeanLanguage, Record<string, string>> = { fr: {}, de: {}, it: {}, pt: {} }
for (const row of rows) {
  if (row.length !== 5 || row.some(value => !value)) throw new Error('Invalid translation row: ' + row[0])
  ;(['fr', 'de', 'it', 'pt'] as const).forEach((code, index) => { european[code][row[0]] = row[index + 1] })
}
// Reuse capitalization and icon variants, while keeping exact English keys.
for (const dictionary of Object.values(european)) {
  const lower = new Map(Object.entries(dictionary).map(([key, value]) => [key.toLowerCase(), value]))
  for (const key of Object.keys(spanish)) {
    if (dictionary[key]) continue
    const matching = lower.get(key.toLowerCase())
    if (matching) {
      dictionary[key] = key === key.toUpperCase() ? matching.toUpperCase() : /^[a-z]/.test(key) ? matching[0].toLowerCase() + matching.slice(1) : matching
      continue
    }
    const decorated = key.match(/^(○ |● |⚠ |✓ |! | · )(.+)$/)
    if (decorated) {
      const value = lower.get(decorated[2].toLowerCase())
      if (value) dictionary[key] = decorated[1] + value
    }
    const ending = key.match(/^(.+?)( \(| ·)$/)
    if (ending) {
      const value = lower.get(ending[1].toLowerCase())
      if (value) dictionary[key] = value + ending[2]
    }
  }
}
