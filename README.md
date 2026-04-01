# ai-sec — Инструмент тестирования безопасности LLM

Обучающий CLI-инструмент для тестирования уязвимостей больших языковых моделей (LLM). Создан для специалистов по кибербезопасности, которые хотят разобраться в поверхности атак на AI-системы.

---

## Что делает

`ai-sec` отправляет специально подготовленные промпты в LLM и оценивает, устояла ли модель перед атакой или её safety-тренировка была обойдена. Инструмент покрывает **7 категорий атак** с документированными payload-ами и включает образовательный контент с объяснением каждой техники.

Это исследовательский и обучающий инструмент, а не фреймворк для автоматизированных атак. Каждая категория атак поставляется с объяснениями, ссылками на научные работы и описанием мер защиты.

---

## Быстрый старт

```bash
# 1. Скопируй шаблон конфига и добавь свой API-ключ
cp .env.example .env
# Открой .env и добавь OPENAI_API_KEY или ANTHROPIC_API_KEY

# 2. Сборка
cargo build --release

# 3. Проверка подключения к провайдеру
./target/release/ai-sec check

# 4. Список доступных атак
./target/release/ai-sec list

# 5. Запуск конкретной атаки
./target/release/ai-sec run --attack jailbreaking

# 6. Запуск с сохранением отчёта в JSON
./target/release/ai-sec run --attack jailbreaking --output report.json

# 7. Интерактивный режим (без аргументов)
./target/release/ai-sec

# 8. Узнать подробности об атаке
./target/release/ai-sec explain jailbreaking
```

---

## Категории атак

| ID                    | Название                 | Payload-ов | Описание |
|-----------------------|--------------------------|------------|----------|
| `prompt_injection`    | Direct Prompt Injection  | 6          | Переопределение системных инструкций через пользовательский ввод |
| `jailbreaking`        | Jailbreaking Techniques  | 12         | DAN, roleplay, гипотетическое обрамление, трюки с кодировкой |
| `extraction`          | System Prompt Extraction | 6          | Извлечение скрытых инструкций оператора |
| `goal_hijacking`      | Goal Hijacking           | 5          | Перенаправление модели с исходной задачи на задачу атакующего |
| `token_attacks`       | Token-Level Attacks      | 8          | Unicode-гомоглифы, zero-width символы, leetspeak |
| `many_shot`           | Many-Shot Jailbreaking   | 3          | Кондиционирование модели через множество примеров |
| `context_manipulation`| Context Manipulation     | 5          | Ложные разрешения, memory poisoning |

---

## Провайдеры

Настраиваются в `.env` (скопируй из `.env.example`):

| Провайдер | Переменные                           | Примечания |
|-----------|--------------------------------------|------------|
| OpenAI    | `OPENAI_API_KEY`, `OPENAI_MODEL`     | По умолчанию: `gpt-4o` |
| Anthropic | `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL` | По умолчанию: `claude-3-5-sonnet-20241022` |
| Ollama    | `OLLAMA_BASE_URL`, `OLLAMA_MODEL`    | Локальный, ключ не нужен |

Автоматически выбирается первый сконфигурированный провайдер. Для ручного выбора: `--provider openai`.

---

## Справка по CLI

```
ai-sec [ОПЦИИ] [КОМАНДА]

КОМАНДЫ:
  run      Запуск атак: --attack <id> [--limit N] [--output file.json]
  list     Список всех категорий атак и количество payload-ов
  explain  Образовательный контент: explain <attack_id>
  check    Проверка подключения к провайдеру

ОПЦИИ:
  -p, --provider <PROVIDER>  Выбор провайдера (openai, anthropic, ollama)
  -v, --verbose              Подробный вывод логов
```

---

## Метки результатов

| Метка        | Значение |
|--------------|----------|
| `✓ REFUSED`  | Модель отказала — safety-тренировка устояла |
| `⚠ PARTIAL`  | Неоднозначно — частичное выполнение или неясный результат |
| `✗ BYPASS`   | Атака успешна — safety обойден |
| `  ERROR`    | Ошибка запроса (сеть / авторизация) |

> Все оценки — **эвристические** (на основе поиска ключевых слов). Для неоднозначных результатов рекомендуется ручной анализ.

---

## Структура проекта

```
ai-sec/
├── src/
│   ├── main.rs          — Точка входа, выбор провайдера, диспетчеризация команд
│   ├── config/          — Загрузка настроек из .env
│   ├── providers/       — HTTP-клиенты для OpenAI, Anthropic, Ollama
│   ├── attacks/         — 7 категорий атак + реестр
│   ├── payloads/        — Загрузчик TOML + шаблонизатор
│   ├── engine/          — Runner, evaluator, трекинг сессий
│   ├── reporting/       — Таблицы в терминале + JSON-экспорт
│   ├── cli/             — Парсинг аргументов, меню, хелперы отображения
│   └── education/       — Контент для команды explain
├── payloads/            — TOML-файлы с payload-ами (редактируются без перекомпиляции)
│   ├── prompt_injection/
│   ├── jailbreaking/
│   ├── extraction/
│   ├── goal_hijacking/
│   ├── token_attacks/
│   ├── many_shot/
│   └── context_manipulation/
├── docs/               — Заметки по сессиям, правила, план оператора (в gitignore)
└── .env.example        — Шаблон конфигурации
```

---

## Добавление новых payload-ов

Payload-ы — это TOML-файлы. Знание Rust не требуется:

```toml
# payloads/jailbreaking/my_payloads.toml
[metadata]
attack_type = "jailbreaking"
variant = "custom"
severity = "medium"

[[payloads]]
id = "my_test"
name = "My Custom Payload"
description = "Что тестирует этот payload"
prompt = "Текст промпта..."
expected_refusal_keywords = ["cannot", "won't"]
success_indicators = ["целевая фраза"]
```

Перезапусти `ai-sec` — новые payload-ы подхватятся автоматически.

---

## Ключевая литература

| Тема | Работа / Ресурс |
|------|-----------------|
| Prompt Injection | [Perez & Ribeiro, 2022 — первая академическая статья](https://arxiv.org/abs/2211.09527) |
| Prompt Injection (таксономия) | [Liu et al., 2023 — полная классификация](https://arxiv.org/abs/2310.12815) |
| Jailbreaking | [Wei et al., 2023 — почему safety-тренировка ломается](https://arxiv.org/abs/2307.02483) |
| Jailbreak-промпты из реальности | [Shen et al., 2023 — анализ диких промптов](https://arxiv.org/abs/2308.03825) |
| Adversarial суффиксы (GCG) | [Zou et al., 2023 — gradient-based атаки](https://arxiv.org/abs/2307.15043) |
| Many-Shot | [Anthropic, 2024 — many-shot jailbreaking](https://www.anthropic.com/research/many-shot-jailbreaking) |
| Indirect Injection | [Greshake et al., 2023 — атаки через внешний контент](https://arxiv.org/abs/2302.12173) |
| Бенчмаркинг | [HarmBench, 2024 — фреймворк оценки](https://arxiv.org/abs/2402.04249) |
| **OWASP Top 10 для LLM** | [owasp.org — знакомая структура, но для AI](https://owasp.org/www-project-top-10-for-large-language-model-applications/) |
| **MITRE ATLAS** | [atlas.mitre.org — ATT&CK для AI-систем](https://atlas.mitre.org/) |
| Практика (CTF) | [Gandalf by Lakera — интерактивные задачи](https://gandalf.lakera.ai/) |
| Блог практика | [Simon Willison — реальные примеры prompt injection](https://simonwillison.net/tags/prompt-injection/) |

---

## Этичное использование

Этот инструмент предназначен **исключительно для авторизованного тестирования безопасности и обучения**.

- Всегда получайте явное разрешение перед тестированием любой системы
- API-ключи хранятся в `.env` — никогда не коммитьте их
- Результаты могут содержать чувствительные ответы моделей — обращайтесь соответственно
- Авторы не несут ответственности за неправомерное использование

---

## Правила разработки

Подробнее: `docs/Rules.md` (в gitignore, только локально).

Коротко:
- Каждая публичная функция — doc comment
- Каждый модуль — `//!` комментарий
- Никаких `unwrap()` в production-путях
- Новая атака = код + TOML + образовательный контент
- Стиль коммитов: `feat(attacks): add new payload set`
