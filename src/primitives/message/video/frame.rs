use crate::primitives::attribute::{Attributive, InnerAttributes};
use crate::primitives::message::video::object::InnerObject;
use crate::primitives::to_json_value::ToSerdeJsonValue;
use crate::primitives::{Attribute, Message, Object};
use pyo3::{pyclass, pymethods, Py, PyAny, PyResult, Python};
use rkyv::{with::Skip, Archive, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[pyclass]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(check_bytes)]
pub struct ExternalFrame {
    #[pyo3(get, set)]
    pub method: String,
    #[pyo3(get, set)]
    pub location: Option<String>,
}

impl ToSerdeJsonValue for ExternalFrame {
    fn to_serde_json_value(&self) -> Value {
        serde_json::json!({
            "method": self.method,
            "location": self.location,
        })
    }
}

#[pymethods]
impl ExternalFrame {
    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    #[new]
    pub fn new(method: String, location: Option<String>) -> Self {
        Self { method, location }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(check_bytes)]
pub enum VideoFrameContent {
    External(ExternalFrame),
    Internal(Vec<u8>),
    None,
}

impl ToSerdeJsonValue for VideoFrameContent {
    fn to_serde_json_value(&self) -> Value {
        match self {
            VideoFrameContent::External(data) => {
                serde_json::json!({"external": data.to_serde_json_value()})
            }
            VideoFrameContent::Internal(_) => {
                serde_json::json!({ "internal": Value::Null })
            }
            VideoFrameContent::None => Value::Null,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyVideoFrameContent {
    pub(crate) inner: VideoFrameContent,
}

impl PyVideoFrameContent {
    pub fn new(inner: VideoFrameContent) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyVideoFrameContent {
    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    #[staticmethod]
    pub fn external(method: String, location: Option<String>) -> Self {
        Self {
            inner: VideoFrameContent::External(ExternalFrame::new(method, location)),
        }
    }

    #[staticmethod]
    pub fn internal(data: Vec<u8>) -> Self {
        Self {
            inner: VideoFrameContent::Internal(data),
        }
    }

    #[staticmethod]
    pub fn none() -> Self {
        Self {
            inner: VideoFrameContent::None,
        }
    }

    #[getter]
    pub fn is_external(&self) -> bool {
        matches!(self.inner, VideoFrameContent::External(_))
    }

    #[getter]
    pub fn is_internal(&self) -> bool {
        matches!(self.inner, VideoFrameContent::Internal(_))
    }

    #[getter]
    pub fn is_none(&self) -> bool {
        matches!(self.inner, VideoFrameContent::None)
    }

    pub fn get_data(&self) -> PyResult<Vec<u8>> {
        match &self.inner {
            VideoFrameContent::Internal(data) => Ok(data.clone()),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Video data is not stored internally",
            )),
        }
    }

    pub fn get_method(&self) -> PyResult<String> {
        match &self.inner {
            VideoFrameContent::External(data) => Ok(data.method.clone()),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Video data is not stored externally",
            )),
        }
    }

    pub fn get_location(&self) -> PyResult<Option<String>> {
        match &self.inner {
            VideoFrameContent::External(data) => Ok(data.location.clone()),
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "Video data is not stored externally",
            )),
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(check_bytes)]
pub enum FrameTransformation {
    InitialSize(u64, u64),
    Scale(u64, u64),
    Padding(u64, u64, u64, u64),
    None,
}

impl ToSerdeJsonValue for FrameTransformation {
    fn to_serde_json_value(&self) -> Value {
        match self {
            FrameTransformation::InitialSize(width, height) => {
                serde_json::json!({"initial_size": [width, height]})
            }
            FrameTransformation::Scale(width, height) => {
                serde_json::json!({"scale": [width, height]})
            }
            FrameTransformation::Padding(left, top, right, bottom) => {
                serde_json::json!({"padding": [left, top, right, bottom]})
            }
            FrameTransformation::None => serde_json::json!(null),
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyFrameTransformation {
    pub(crate) inner: FrameTransformation,
}

impl PyFrameTransformation {
    pub fn new(inner: FrameTransformation) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyFrameTransformation {
    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    #[staticmethod]
    pub fn initial_size(width: i64, height: i64) -> Self {
        assert!(width > 0 && height > 0);
        Self {
            inner: FrameTransformation::InitialSize(
                u64::try_from(width).unwrap(),
                u64::try_from(height).unwrap(),
            ),
        }
    }

    #[staticmethod]
    pub fn scale(width: i64, height: i64) -> Self {
        assert!(width > 0 && height > 0);
        Self {
            inner: FrameTransformation::Scale(
                u64::try_from(width).unwrap(),
                u64::try_from(height).unwrap(),
            ),
        }
    }

    #[staticmethod]
    pub fn padding(left: i64, top: i64, right: i64, bottom: i64) -> Self {
        assert!(left >= 0 && top >= 0 && right >= 0 && bottom >= 0);
        Self {
            inner: FrameTransformation::Padding(
                u64::try_from(left).unwrap(),
                u64::try_from(top).unwrap(),
                u64::try_from(right).unwrap(),
                u64::try_from(bottom).unwrap(),
            ),
        }
    }

    #[staticmethod]
    pub fn none() -> Self {
        Self {
            inner: FrameTransformation::None,
        }
    }

    #[getter]
    pub fn is_initial_size(&self) -> bool {
        matches!(self.inner, FrameTransformation::InitialSize(_, _))
    }

    #[getter]
    pub fn is_scale(&self) -> bool {
        matches!(self.inner, FrameTransformation::Scale(_, _))
    }

    #[getter]
    pub fn is_padding(&self) -> bool {
        matches!(self.inner, FrameTransformation::Padding(_, _, _, _))
    }

    #[getter]
    pub fn is_none(&self) -> bool {
        matches!(self.inner, FrameTransformation::None)
    }

    #[getter]
    pub fn as_initial_size(&self) -> Option<(u64, u64)> {
        match &self.inner {
            FrameTransformation::InitialSize(w, h) => Some((*w, *h)),
            _ => None,
        }
    }

    #[getter]
    pub fn as_scale(&self) -> Option<(u64, u64)> {
        match &self.inner {
            FrameTransformation::Scale(w, h) => Some((*w, *h)),
            _ => None,
        }
    }

    #[getter]
    pub fn as_padding(&self) -> Option<(u64, u64, u64, u64)> {
        match &self.inner {
            FrameTransformation::Padding(l, t, r, b) => Some((*l, *t, *r, *b)),
            _ => None,
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone, derive_builder::Builder)]
#[archive(check_bytes)]
pub struct InnerVideoFrame {
    pub source_id: String,
    pub framerate: String,
    pub width: i64,
    pub height: i64,
    pub codec: Option<String>,
    pub keyframe: Option<bool>,
    pub pts: i64,
    pub dts: Option<i64>,
    pub duration: Option<i64>,
    pub content: VideoFrameContent,
    pub transformations: Vec<FrameTransformation>,
    pub attributes: HashMap<(String, String), Attribute>,
    pub offline_objects: Vec<InnerObject>,
    #[with(Skip)]
    pub(crate) resident_objects: Vec<Arc<Mutex<InnerObject>>>,
}

impl ToSerdeJsonValue for InnerVideoFrame {
    fn to_serde_json_value(&self) -> Value {
        serde_json::json!(
            {
                "type": "VideoFrame",
                "source_id": self.source_id,
                "framerate": self.framerate,
                "width": self.width,
                "height": self.height,
                "codec": self.codec,
                "keyframe": self.keyframe,
                "pts": self.pts,
                "dts": self.dts,
                "duration": self.duration,
                "content": self.content.to_serde_json_value(),
                "transformations": self.transformations.iter().map(|t| t.to_serde_json_value()).collect::<Vec<_>>(),
                "attributes": self.attributes.values().map(|v| v.to_serde_json_value()).collect::<Vec<_>>(),
                "objects": self.resident_objects.iter().map(|o| o.lock().unwrap().to_serde_json_value()).collect::<Vec<_>>(),
            }
        )
    }
}

impl InnerAttributes for Box<InnerVideoFrame> {
    fn get_attributes_ref(&self) -> &HashMap<(String, String), Attribute> {
        &self.attributes
    }

    fn get_attributes_ref_mut(&mut self) -> &mut HashMap<(String, String), Attribute> {
        &mut self.attributes
    }
}

impl InnerVideoFrame {
    pub(crate) fn prepare_before_save(&mut self) {
        self.offline_objects = self
            .resident_objects
            .iter()
            .map(|o| o.lock().unwrap().clone())
            .collect();
    }

    pub(crate) fn prepare_after_load(&mut self) {
        self.resident_objects = self
            .offline_objects
            .iter()
            .map(|o| Arc::new(Mutex::new(o.clone())))
            .collect();
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub(crate) inner: Arc<Mutex<Box<InnerVideoFrame>>>,
}

impl Attributive<Box<InnerVideoFrame>> for VideoFrame {
    fn get_inner(&self) -> Arc<Mutex<Box<InnerVideoFrame>>> {
        self.inner.clone()
    }
}

impl VideoFrame {
    pub(crate) fn from_inner(object: InnerVideoFrame) -> Self {
        VideoFrame {
            inner: Arc::new(Mutex::new(Box::new(object))),
        }
    }
}

impl ToSerdeJsonValue for VideoFrame {
    fn to_serde_json_value(&self) -> Value {
        self.inner.lock().unwrap().to_serde_json_value()
    }
}

#[pymethods]
impl VideoFrame {
    #[classattr]
    const __hash__: Option<Py<PyAny>> = None;

    fn __repr__(&self) -> String {
        format!("{:#?}", self.inner.lock().unwrap())
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(
        signature = (source_id, framerate, width, height, content, codec=None, keyframe=None, pts=0, dts=None, duration=None)
    )]
    pub fn new(
        source_id: String,
        framerate: String,
        width: i64,
        height: i64,
        content: PyVideoFrameContent,
        codec: Option<String>,
        keyframe: Option<bool>,
        pts: i64,
        dts: Option<i64>,
        duration: Option<i64>,
    ) -> Self {
        VideoFrame::from_inner(InnerVideoFrame {
            source_id,
            pts,
            framerate,
            width,
            height,
            dts,
            duration,
            codec,
            keyframe,
            transformations: vec![],
            content: content.inner,
            attributes: HashMap::default(),
            offline_objects: vec![],
            resident_objects: vec![],
        })
    }

    pub fn to_message(&self) -> Message {
        Message::video_frame(self.clone())
    }

    #[getter]
    pub fn get_source_id(&self) -> String {
        self.inner.lock().unwrap().source_id.clone()
    }

    #[getter]
    pub fn get_json(&self) -> String {
        serde_json::to_string(&self.to_serde_json_value()).unwrap()
    }

    #[setter]
    pub fn set_source_id(&mut self, source_id: String) {
        let mut frame = self.inner.lock().unwrap();
        frame.source_id = source_id;
    }

    #[getter]
    pub fn get_pts(&self) -> i64 {
        self.inner.lock().unwrap().pts
    }

    #[setter]
    pub fn set_pts(&mut self, pts: i64) {
        assert!(pts >= 0, "pts must be greater than or equal to 0");
        let mut frame = self.inner.lock().unwrap();
        frame.pts = pts;
    }

    #[getter]
    pub fn get_framerate(&self) -> String {
        self.inner.lock().unwrap().framerate.clone()
    }

    #[setter]
    pub fn set_framerate(&mut self, framerate: String) {
        let mut frame = self.inner.lock().unwrap();
        frame.framerate = framerate;
    }

    #[getter]
    pub fn get_width(&self) -> i64 {
        self.inner.lock().unwrap().width
    }

    #[setter]
    pub fn set_width(&mut self, width: i64) {
        assert!(width > 0, "width must be greater than 0");
        let mut frame = self.inner.lock().unwrap();
        frame.width = width;
    }

    #[getter]
    pub fn get_height(&self) -> i64 {
        self.inner.lock().unwrap().height
    }

    #[setter]
    pub fn set_height(&mut self, height: i64) {
        assert!(height > 0, "height must be greater than 0");
        let mut frame = self.inner.lock().unwrap();
        frame.height = height;
    }

    #[getter]
    pub fn get_dts(&self) -> Option<i64> {
        let frame = self.inner.lock().unwrap();
        frame.dts
    }

    #[setter]
    pub fn set_dts(&mut self, dts: Option<i64>) {
        assert!(
            dts.is_none() || dts.unwrap() >= 0,
            "dts must be greater than or equal to 0"
        );
        let mut frame = self.inner.lock().unwrap();
        frame.dts = dts;
    }

    #[getter]
    pub fn get_duration(&self) -> Option<i64> {
        let frame = self.inner.lock().unwrap();
        frame.duration
    }

    #[setter]
    pub fn set_duration(&mut self, duration: Option<i64>) {
        assert!(
            duration.is_none() || duration.unwrap() >= 0,
            "duration must be greater than or equal to 0"
        );
        let mut frame = self.inner.lock().unwrap();
        frame.duration = duration;
    }

    #[getter]
    pub fn get_codec(&self) -> Option<String> {
        let frame = self.inner.lock().unwrap();
        frame.codec.clone()
    }

    #[setter]
    pub fn set_codec(&mut self, codec: Option<String>) {
        let mut frame = self.inner.lock().unwrap();
        frame.codec = codec;
    }

    pub fn clear_transformations(&mut self) {
        let mut frame = self.inner.lock().unwrap();
        frame.transformations.clear();
    }

    pub fn add_transformation(&mut self, transformation: PyFrameTransformation) {
        let mut frame = self.inner.lock().unwrap();
        frame.transformations.push(transformation.inner);
    }

    #[getter]
    pub fn get_transformations(&self) -> Vec<PyFrameTransformation> {
        let frame = self.inner.lock().unwrap();
        frame
            .transformations
            .iter()
            .map(|t| PyFrameTransformation::new(t.clone()))
            .collect()
    }

    #[getter]
    pub fn get_keyframe(&self) -> Option<bool> {
        let frame = self.inner.lock().unwrap();
        frame.keyframe
    }

    #[setter]
    pub fn set_keyframe(&mut self, keyframe: Option<bool>) {
        let mut frame = self.inner.lock().unwrap();
        frame.keyframe = keyframe;
    }

    #[getter]
    pub fn get_content(&self) -> PyVideoFrameContent {
        let frame = self.inner.lock().unwrap();
        PyVideoFrameContent::new(frame.content.clone())
    }

    #[setter]
    pub fn set_content(&mut self, content: PyVideoFrameContent) {
        let mut frame = self.inner.lock().unwrap();
        frame.content = content.inner;
    }

    #[getter]
    pub fn attributes(&self) -> Vec<(String, String)> {
        Python::with_gil(move |py| py.allow_threads(move || self.get_attributes()))
    }

    #[pyo3(name = "find_attributes")]
    pub fn find_attributes_py(
        &self,
        creator: Option<String>,
        name: Option<String>,
        hint: Option<String>,
    ) -> Vec<(String, String)> {
        Python::with_gil(move |py| {
            py.allow_threads(move || self.find_attributes(creator, name, hint))
        })
    }

    #[pyo3(name = "get_attribute")]
    pub fn get_attribute_py(&self, creator: String, name: String) -> Option<Attribute> {
        self.get_attribute(creator, name)
    }

    #[pyo3(signature = (negated=false, creator=None, names=vec![]))]
    #[pyo3(name = "delete_attributes")]
    pub fn delete_attributes_py(
        &mut self,
        negated: bool,
        creator: Option<String>,
        names: Vec<String>,
    ) {
        Python::with_gil(move |py| {
            py.allow_threads(move || self.delete_attributes(negated, creator, names))
        })
    }

    #[pyo3(name = "delete_attribute")]
    pub fn delete_attribute_py(&mut self, creator: String, name: String) -> Option<Attribute> {
        self.delete_attribute(creator, name)
    }

    #[pyo3(name = "set_attribute")]
    pub fn set_attribute_py(&mut self, attribute: Attribute) -> Option<Attribute> {
        self.set_attribute(attribute)
    }

    #[pyo3(name = "clear_attributes")]
    pub fn clear_attributes_py(&mut self) {
        self.clear_attributes()
    }

    pub fn get_object(&self, id: i64) -> Option<Object> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let frame = self.inner.lock().unwrap();
                frame
                    .resident_objects
                    .iter()
                    .find(|o| o.lock().unwrap().id == id)
                    .map(|o| Object::from_arc_inner_object(o.clone()))
            })
        })
    }

    pub fn access_objects(
        &self,
        negated: bool,
        creator: Option<String>,
        label: Option<String>,
    ) -> Vec<Object> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let frame = self.inner.lock().unwrap();
                frame
                    .resident_objects
                    .iter()
                    .filter(|o| {
                        let o = o.lock().unwrap();
                        let creator_match = match &creator {
                            Some(creator) => o.creator == *creator,
                            None => true,
                        };
                        let label_match = match &label {
                            Some(label) => o.label == *label,
                            None => true,
                        };
                        (creator_match && label_match) ^ negated
                    })
                    .map(|o| Object::from_arc_inner_object(o.clone()))
                    .collect()
            })
        })
    }

    pub fn access_objects_by_id(&self, ids: Vec<i64>) -> Vec<Object> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let frame = self.inner.lock().unwrap();
                frame
                    .resident_objects
                    .iter()
                    .filter(|o| ids.contains(&o.lock().unwrap().id))
                    .map(|o| Object::from_arc_inner_object(o.clone()))
                    .collect()
            })
        })
    }

    pub fn add_object(&mut self, object: Object) {
        let mut frame = self.inner.lock().unwrap();
        frame.resident_objects.push(object.inner);
    }

    pub fn delete_objects_by_ids(&mut self, ids: Vec<i64>) {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let mut frame = self.inner.lock().unwrap();
                frame
                    .resident_objects
                    .retain(|o| !ids.contains(&o.lock().unwrap().id));
            })
        })
    }

    pub fn delete_objects(
        &mut self,
        negated: bool,
        creator: Option<String>,
        label: Option<String>,
    ) {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let mut frame = self.inner.lock().unwrap();
                frame.resident_objects.retain(|o| {
                    let o = o.lock().unwrap();
                    let creator_match = match &creator {
                        Some(creator) => o.creator == *creator,
                        None => true,
                    };
                    let label_match = match &label {
                        Some(label) => o.label == *label,
                        None => true,
                    };
                    !((creator_match && label_match) ^ negated)
                });
            })
        })
    }

    pub fn clear_objects(&mut self) {
        let mut frame = self.inner.lock().unwrap();
        frame.resident_objects.clear();
    }

    pub fn snapshot(&mut self) {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let mut frame = self.inner.lock().unwrap();
                frame.prepare_before_save();
            })
        })
    }

    pub fn restore(&mut self) {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let mut frame = self.inner.lock().unwrap();
                frame.resident_objects.clear();
                frame.prepare_after_load();
            })
        })
    }

    pub fn get_modified_objects(&self) -> Vec<Object> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                let frame = self.inner.lock().unwrap();
                frame
                    .resident_objects
                    .iter()
                    .filter(|o| !o.lock().unwrap().modifications.is_empty())
                    .map(|o| Object::from_arc_inner_object(o.clone()))
                    .collect()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::attribute::Attributive;
    use crate::primitives::Modification;
    use crate::test::utils::gen_frame;

    #[test]
    fn test_access_objects_by_id() {
        pyo3::prepare_freethreaded_python();
        let t = gen_frame();
        let objects = t.access_objects_by_id(vec![0, 1]);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 0);
        assert_eq!(objects[1].get_id(), 1);
    }

    #[test]
    fn test_access_objects() {
        pyo3::prepare_freethreaded_python();

        let t = gen_frame();
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 3);

        let t = gen_frame();
        let objects = t.access_objects(true, None, None);
        assert!(objects.is_empty());

        let t = gen_frame();
        let objects = t.access_objects(false, Some("abc".to_string()), None);
        assert!(objects.is_empty());

        let t = gen_frame();
        let objects = t.access_objects(true, Some("abc".to_string()), None);
        assert_eq!(objects.len(), 3);

        let t = gen_frame();
        let objects = t.access_objects(false, Some("test2".to_string()), None);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 1);
        assert_eq!(objects[1].get_id(), 2);

        let t = gen_frame();
        let objects = t.access_objects(true, Some("test2".to_string()), None);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].get_id(), 0);

        let t = gen_frame();
        let objects = t.access_objects(false, Some("test2".to_string()), Some("test2".to_string()));
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].get_id(), 2);

        let t = gen_frame();
        let objects = t.access_objects(true, Some("test2".to_string()), Some("test2".to_string()));
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 0);
        assert_eq!(objects[1].get_id(), 1);
    }

    #[test]
    fn test_objects_by_id() {
        pyo3::prepare_freethreaded_python();
        let t = gen_frame();
        let objects = t.access_objects_by_id(vec![0, 1]);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 0);
        assert_eq!(objects[1].get_id(), 1);
    }

    #[test]
    fn test_get_attribute() {
        pyo3::prepare_freethreaded_python();
        let t = gen_frame();
        let attribute = t.get_attribute("system".to_string(), "test".to_string());
        assert!(attribute.is_some());
        assert_eq!(
            attribute.unwrap().values[0].as_string().unwrap(),
            "1".to_string()
        );
    }

    #[test]
    fn test_find_attributes() {
        pyo3::prepare_freethreaded_python();
        let t = gen_frame();
        let mut attributes = t.find_attributes_py(Some("system".to_string()), None, None);
        attributes.sort();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0], ("system".to_string(), "test".to_string()));
        assert_eq!(attributes[1], ("system".to_string(), "test2".to_string()));

        let attributes =
            t.find_attributes_py(Some("system".to_string()), Some("test".to_string()), None);
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0], ("system".to_string(), "test".to_string()));

        let attributes = t.find_attributes_py(
            Some("system".to_string()),
            Some("test".to_string()),
            Some("test".to_string()),
        );
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0], ("system".to_string(), "test".to_string()));

        let mut attributes = t.find_attributes_py(None, None, Some("test".to_string()));
        attributes.sort();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0], ("system".to_string(), "test".to_string()));
        assert_eq!(attributes[1], ("system".to_string(), "test2".to_string()));
    }

    #[test]
    fn test_delete_objects_by_ids() {
        pyo3::prepare_freethreaded_python();
        let mut t = gen_frame();
        t.delete_objects_by_ids(vec![0, 1]);
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].get_id(), 2);
    }

    #[test]
    fn test_delete_objects() {
        pyo3::prepare_freethreaded_python();
        let mut t = gen_frame();
        t.delete_objects(false, None, None);
        let objects = t.access_objects(false, None, None);
        assert!(objects.is_empty());

        let mut t = gen_frame();
        t.delete_objects(true, None, None);
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 3);

        let mut t = gen_frame();
        t.delete_objects(false, Some("test2".to_string()), None);
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].get_id(), 0);

        let mut t = gen_frame();
        t.delete_objects(true, Some("test2".to_string()), None);
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 1);
        assert_eq!(objects[1].get_id(), 2);

        let mut t = gen_frame();
        t.delete_objects(false, Some("test2".to_string()), Some("test2".to_string()));
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].get_id(), 0);
        assert_eq!(objects[1].get_id(), 1);

        let mut t = gen_frame();
        t.delete_objects(true, Some("test2".to_string()), Some("test2".to_string()));
        let objects = t.access_objects(false, None, None);
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].get_id(), 2);
    }

    #[test]
    fn test_snapshotting() {
        let mut t = gen_frame();
        t.snapshot();
        let mut o = t.access_objects_by_id(vec![0]).pop().unwrap();
        o.set_id(12);
        assert!(matches!(t.access_objects_by_id(vec![0]).pop(), None));
        t.restore();
        t.access_objects_by_id(vec![0]).pop().unwrap();
    }

    #[test]
    fn test_modified_objects() {
        let t = gen_frame();
        let mut o = t.access_objects_by_id(vec![0]).pop().unwrap();
        o.set_id(12);
        let mut modified = t.get_modified_objects();
        assert_eq!(modified.len(), 1);
        let modified = modified.pop().unwrap();
        assert_eq!(modified.get_id(), 12);

        let mods = modified.take_modifications();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods, vec![Modification::Id]);

        let modified = t.get_modified_objects();
        assert!(modified.is_empty());
    }
}
