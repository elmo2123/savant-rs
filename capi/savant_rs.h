#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define NATIVE_MESSAGE_MARKER_LEN 4

typedef struct InferenceObjectMeta {
  int64_t id;
  int64_t creator_id;
  int64_t label_id;
  double confidence;
  int64_t track_id;
  int64_t parent_id;
  double box_xc;
  double box_yx;
  double box_width;
  double box_height;
  double box_angle;
} InferenceObjectMeta;

/**
 * Returns the object vector length
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer
 *
 */
uintptr_t object_vector_len(uintptr_t handle);

/**
 * Returns the object data casted to InferenceObjectMeta by index
 *
 * # Safety
 *
 * This function is unsafe because it dereferences a raw pointer
 *
 */
struct InferenceObjectMeta get_inference_meta(uintptr_t handle, uintptr_t pos);
