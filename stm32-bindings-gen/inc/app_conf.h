/* Stub configuration header used during bindings generation. */

#ifndef STM32_BINDINGS_GEN_APP_CONF_H
#define STM32_BINDINGS_GEN_APP_CONF_H

#ifdef __cplusplus
extern "C" {
#endif

/* Provide minimal definitions so that ST middleware headers compile under bindgen. */

#ifndef LOG_LEVEL_INFO
#define LOG_LEVEL_INFO (0)
#endif

#ifndef LOG_INFO_APP
#define LOG_INFO_APP(...)
#endif

#ifndef CFG_LOG_SUPPORTED
#define CFG_LOG_SUPPORTED (0U)
#endif

#ifndef PWR_LDO_SUPPLY
#define PWR_LDO_SUPPLY (0U)
#endif

#ifndef RADIO_INTR_NUM
#define RADIO_INTR_NUM (0U)
#endif

#ifndef RADIO_INTR_PRIO_HIGH
#define RADIO_INTR_PRIO_HIGH (0U)
#endif

#ifndef RADIO_INTR_PRIO_LOW
#define RADIO_INTR_PRIO_LOW (0U)
#endif

#ifdef __cplusplus
}
#endif

#endif /* STM32_BINDINGS_GEN_APP_CONF_H */
