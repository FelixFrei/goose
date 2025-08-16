# Requirements Document

## Introduction

Dieses Feature fügt einen neuen Model-Provider für die Swiss AI Platform zu Goose hinzu. Der Provider soll Llama-3.3 und Llama-4 Modelle unterstützen und eine OpenAI-kompatible API verwenden. Die Implementation basiert auf der bestehenden Groq-Provider-Struktur als Vorlage.

## Requirements

### Requirement 1

**User Story:** Als Entwickler möchte ich die Swiss AI Platform als Provider in Goose konfigurieren können, damit ich Llama-3.3 und Llama-4 Modelle verwenden kann.

#### Acceptance Criteria

1. WHEN der Benutzer "swiss-ai" als Provider konfiguriert THEN soll das System eine Verbindung zur Swiss AI Platform API herstellen
2. WHEN die Konfiguration gültige API-Credentials enthält THEN soll der Provider erfolgreich initialisiert werden
3. IF keine API-Credentials vorhanden sind THEN soll das System eine aussagekräftige Fehlermeldung anzeigen
4. WHEN der Provider initialisiert wird THEN soll er die Standard-Konfiguration für Swiss AI Platform verwenden

### Requirement 2

**User Story:** Als Benutzer möchte ich Llama-3.3 und Llama-4 Modelle über die Swiss AI Platform verwenden können, damit ich Zugang zu diesen spezifischen Modellen habe.

#### Acceptance Criteria

1. WHEN der Swiss AI Provider geladen wird THEN soll er "llama-3.3-70b-instruct" und "llama-4-405b-instruct" als verfügbare Modelle anbieten
2. WHEN ein Chat-Request mit einem Swiss AI Modell gesendet wird THEN soll die Anfrage an die Swiss AI Platform API weitergeleitet werden
3. WHEN die API eine gültige Antwort zurückgibt THEN soll diese korrekt als Message-Objekt geparst werden
4. WHEN die API Usage-Informationen zurückgibt THEN sollen diese korrekt erfasst und weitergegeben werden

### Requirement 3

**User Story:** Als Entwickler möchte ich, dass der Swiss AI Provider OpenAI-kompatible API-Calls verwendet, damit die Integration nahtlos funktioniert.

#### Acceptance Criteria

1. WHEN der Provider eine Chat-Completion-Anfrage sendet THEN soll er das OpenAI-kompatible Format verwenden
2. WHEN Tools in der Anfrage enthalten sind THEN sollen diese im OpenAI-Format serialisiert werden
3. WHEN die API-Antwort empfangen wird THEN soll sie mit den bestehenden OpenAI-kompatiblen Parsing-Funktionen verarbeitet werden
4. WHEN Bilder in Nachrichten enthalten sind THEN sollen diese im OpenAI-Format übertragen werden

### Requirement 4

**User Story:** Als Administrator möchte ich die Swiss AI Platform Konfiguration über Umgebungsvariablen verwalten können, damit die Credentials sicher gespeichert werden.

#### Acceptance Criteria

1. WHEN der Provider initialisiert wird THEN soll er die Umgebungsvariable "SWISS_AI_API_KEY" für die Authentifizierung verwenden
2. WHEN eine benutzerdefinierte Host-URL konfiguriert ist THEN soll die Umgebungsvariable "SWISS_AI_HOST" verwendet werden
3. IF keine Host-URL konfiguriert ist THEN soll eine Standard-URL verwendet werden
4. WHEN die Konfiguration abgerufen wird THEN sollen die erforderlichen und optionalen Parameter korrekt dokumentiert sein

### Requirement 5

**User Story:** Als Benutzer möchte ich verfügbare Modelle von der Swiss AI Platform dynamisch abrufen können, damit ich immer die aktuellsten verfügbaren Modelle sehe.

#### Acceptance Criteria

1. WHEN der Provider nach verfügbaren Modellen gefragt wird THEN soll er eine API-Anfrage an den "/v1/models" Endpoint senden
2. WHEN die Modell-Liste erfolgreich abgerufen wird THEN soll sie als sortierte Liste von Modell-Namen zurückgegeben werden
3. IF die API-Anfrage fehlschlägt THEN soll ein entsprechender ProviderError zurückgegeben werden
4. WHEN keine Modelle gefunden werden THEN soll Ok(None) zurückgegeben werden

### Requirement 6

**User Story:** Als Entwickler möchte ich, dass der Swiss AI Provider in die bestehende Provider-Factory integriert ist, damit er über das Standard-Interface verwendet werden kann.

#### Acceptance Criteria

1. WHEN die Provider-Factory initialisiert wird THEN soll der Swiss AI Provider in der Liste der verfügbaren Provider enthalten sein
2. WHEN "swiss-ai" als Provider-Name angegeben wird THEN soll die Factory eine Swiss AI Provider-Instanz erstellen
3. WHEN die Provider-Metadaten abgerufen werden THEN sollen sie korrekte Informationen über Swiss AI Platform enthalten
4. WHEN der Provider in der Dokumentation aufgelistet wird THEN soll er mit korrekten Metadaten erscheinen

### Requirement 7

**User Story:** Als Benutzer möchte ich aussagekräftige Fehlermeldungen erhalten, wenn Probleme mit der Swiss AI Platform auftreten, damit ich diese beheben kann.

#### Acceptance Criteria

1. WHEN die API-Authentifizierung fehlschlägt THEN soll eine klare Fehlermeldung über ungültige Credentials angezeigt werden
2. WHEN die API nicht erreichbar ist THEN soll eine Netzwerk-Fehlermeldung angezeigt werden
3. WHEN ein Modell nicht verfügbar ist THEN soll eine spezifische Fehlermeldung über das nicht verfügbare Modell angezeigt werden
4. WHEN Rate-Limits erreicht werden THEN soll der Provider automatisch Retry-Mechanismen verwenden

### Requirement 8

**User Story:** Als Entwickler möchte ich, dass der Swiss AI Provider die gleichen Retry- und Error-Handling-Mechanismen wie andere Provider verwendet, damit das Verhalten konsistent ist.

#### Acceptance Criteria

1. WHEN temporäre Netzwerkfehler auftreten THEN soll der Provider automatisch Retry-Versuche unternehmen
2. WHEN die maximale Anzahl von Retries erreicht wird THEN soll ein finaler Fehler zurückgegeben werden
3. WHEN Rate-Limiting auftritt THEN soll der Provider mit exponential backoff retry
4. WHEN permanente Fehler auftreten THEN soll sofort ein Fehler zurückgegeben werden ohne Retry