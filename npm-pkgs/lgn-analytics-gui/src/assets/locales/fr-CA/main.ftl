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
global-timeline = Ligne du temps
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
log-parent-link = { global-log } du processus parent

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
