# Global
global-pagination-first = Première
global-pagination-previous = Précédente
global-pagination-last = Dernière
global-pagination-next = Suivante
global-platform =
  {$platform ->
    [linux] Linux
    [windows] Windows
    *[unknown] Inconnue
  }
global-link-copy = Copier le lien
global-link-share = Partager le lien
global-cumulative-call-graph = Graphique d'appel cumulatif
global-log = Journal
global-timeline = Fils d'exécution
global-metrics = Mesures
global-thread = unité

# Process list
process-list-user = Utilisateur
process-list-process = Processus
process-list-computer = Machine
process-list-platform = Plateforme
process-list-start-time = Débuté à
process-list-statistics = Statistiques
process-list-search = Rechercher des processus...

# Log
log-process-id = Identifiant du processus :
log-executable = Nom de l'exécutable :
log-parent-link = Parent
log-parent-table-column =
  {$columnName ->
    [level] Sévérité
    [timeMs] Horodatage
    [target] Cible
    [msg] Message
    *[other] Inconnue
  }
global-severity-level =
  {$level ->
    [0] Error
    [1] Warn
    [2] Info
    [3] Debug
    [4] Trace
    *[other] Unknown
  }
log-search = Rechercher dans le journal...

# Timeline
timeline-open-cumulative-call-graph = Ouvrir le { LOWERCASE(global-cumulative-call-graph) }
timeline-search = Rechercher...
timeline-table-function = Fonction
timeline-table-count = Total
timeline-table-average = Moyenne
timeline-table-minimum = Min
timeline-table-maximum = Max
timeline-table-standard-deviation = Écart type
timeline-table-sum = Somme
timeline-main-collapsed-extra =
  {$validThreadCount ->
    [0] (Aucune donnée disponible)
    *[other] ({$validThreadCount} {$validThreadCount ->
      [one] unité
      *[other] unités
    } contenant des données)
  }
timeline-main-thread-description-title =
  {$threadName}
  {$threadLength}
  {$threadBlocks} {$threadBlocks ->
    [one] bloc
    *[other] blocs
  }
timeline-main-thread-description =
  {$threadLength} ({$threadBlocks} {$threadBlocks ->
    [one] bloc
    *[other] blocs
  })
timeline-main-collapse = Fermer
timeline-main-expand = Ouvrir
timeline-debug-tooltip =
  Un pixel représente : { $pixelSize }
  Niveau de détail : { $lod }
  Seuil : { $threshold }
  Nombre d'événements : { $events }
timeline-link-copy-notification-title = Copie effectuée
timeline-link-copy-notification-message = Le lien a bien été copié dans votre presse-papier

# Metrics
metrics-search-choose-metrics = Filtrer les mesures
metrics-search-placeholder = Rechercher des mesures...
metrics-search-result-number =
  {$selectedMetricCount} {$selectedMetricCount ->
    [one] mesure
    *[other] mesures
  } sélectionnées
metrics-search-clear = Effacer les filtres
metrics-recently-used = Mesures récemment utilisées
metrics-all-metrics = Liste des mesures
metrics-open-cumulative-call-graph = Ouvrir le { LOWERCASE(global-cumulative-call-graph) }
metrics-open-timeline = Ouvrir la page contenant les { LOWERCASE(global-timeline) }
metrics-selected-time-range = Plage temporelle sélectionnée
metrics-selected-time-range-duration = Durée :
metrics-selected-time-range-beginning = Début :
metrics-selected-time-range-end = Fin :
