use crate::base::*;
use crate::resampling::ResampleAlgorithm;
use miniaudio_sys as sys;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::os::raw::c_void;
use std::ptr;
use std::ptr::NonNull;
use std::sync::Arc;

type MADeviceConfigResampling = sys::ma_device_config__bindgen_ty_1;
type MADeviceConfigPlayback = sys::ma_device_config__bindgen_ty_2;
type MADeviceConfigCapture = sys::ma_device_config__bindgen_ty_3;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    WASAPI = sys::ma_backend_wasapi as _,
    DSound = sys::ma_backend_dsound as _,
    WinMM = sys::ma_backend_winmm as _,
    CoreAudio = sys::ma_backend_coreaudio as _,
    SNDIO = sys::ma_backend_sndio as _,
    Audio4 = sys::ma_backend_audio4 as _,
    OSS = sys::ma_backend_oss as _,
    PulseAudio = sys::ma_backend_pulseaudio as _,
    ALSA = sys::ma_backend_alsa as _,
    Jack = sys::ma_backend_jack as _,
    AAudio = sys::ma_backend_aaudio as _,
    OpenSL = sys::ma_backend_opensl as _,
    WebAudio = sys::ma_backend_webaudio as _,
    Null = sys::ma_backend_null as _,
}
impl_from_c!(Backend, sys::ma_backend);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadPriority {
    Idle = sys::ma_thread_priority_idle as _,
    Lowest = sys::ma_thread_priority_lowest as _,
    Low = sys::ma_thread_priority_low as _,
    Normal = sys::ma_thread_priority_normal as _,
    High = sys::ma_thread_priority_high as _,
    Highest = sys::ma_thread_priority_highest as _,
    Realtime = sys::ma_thread_priority_realtime as _,
}
impl_from_c!(ThreadPriority, sys::ma_thread_priority);

impl ThreadPriority {
    pub const DEFAULT: ThreadPriority = ThreadPriority::Highest;
}

impl Default for ThreadPriority {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Playback = sys::ma_device_type_playback as _,
    Capture = sys::ma_device_type_capture as _,
    Duplex = sys::ma_device_type_duplex as _,
    Loopback = sys::ma_device_type_loopback as _,
}
impl_from_c!(DeviceType, sys::ma_device_type);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMode {
    Shared = sys::ma_share_mode_shared as _,
    Exclusive = sys::ma_share_mode_exclusive as _,
}
impl_from_c!(ShareMode, sys::ma_share_mode);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOSSessionCategory {
    Default = sys::ma_ios_session_category_default as _,
    None = sys::ma_ios_session_category_none as _,
    Ambient = sys::ma_ios_session_category_ambient as _,
    SoloAmbient = sys::ma_ios_session_category_solo_ambient as _,
    Playback = sys::ma_ios_session_category_playback as _,
    Record = sys::ma_ios_session_category_record as _,
    PlayAndRecord = sys::ma_ios_session_category_play_and_record as _,
    MultiRoute = sys::ma_ios_session_category_multi_route as _,
}
impl_from_c!(IOSSessionCategory, sys::ma_ios_session_category);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOSSessionCategoryOption {
    MixWithOthers = sys::ma_ios_session_category_option_mix_with_others as _,
    DuckOthers = sys::ma_ios_session_category_option_duck_others as _,
    AllowBluetooth = sys::ma_ios_session_category_option_allow_bluetooth as _,
    DefaultToSpeaker = sys::ma_ios_session_category_option_default_to_speaker as _,
    InterruptSpokenAudioAndMixWithOthers =
        sys::ma_ios_session_category_option_interrupt_spoken_audio_and_mix_with_others as _,
    AllowBluetoothA2DP = sys::ma_ios_session_category_option_allow_bluetooth_a2dp as _,
    AllowAirPlay = sys::ma_ios_session_category_option_allow_air_play as _,
}
impl_from_c!(
    IOSSessionCategoryOption,
    sys::ma_ios_session_category_option
);

// FIXME probably going to want a way to compare these???
#[repr(transparent)]
pub struct DeviceId(sys::ma_device_id);

#[repr(transparent)]
pub struct DeviceInfo(sys::ma_device_info);

impl DeviceInfo {
    #[inline]
    pub fn id(&self) -> DeviceId {
        unsafe { std::mem::transmute(self.0.id) }
    }

    #[inline]
    pub fn name(&self) -> &str {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.0.name.as_ptr()) };

        // FIXME at the moment we just return a blank string instead of invalid UTF-8
        cstr.to_str().unwrap_or("")
    }

    #[inline]
    pub fn format_count(&self) -> u32 {
        self.0.formatCount
    }

    #[inline]
    pub fn formats(&self) -> [Format; sys::ma_format_count as usize] {
        // FIXME this should truncate the array and return a slice with length `format_count`
        unsafe { std::mem::transmute(self.0.formats) }
    }

    #[inline]
    pub fn min_channels(&self) -> u32 {
        self.0.minChannels
    }

    #[inline]
    pub fn max_channels(&self) -> u32 {
        self.0.maxChannels
    }

    #[inline]
    pub fn min_sample_rate(&self) -> u32 {
        self.0.minSampleRate
    }

    #[inline]
    pub fn max_sample_rate(&self) -> u32 {
        self.0.maxSampleRate
    }
}

pub type DataCallbackProc = extern "C" fn(
    device: NonNull<Device>,
    output: Option<NonNull<()>>,
    input: Option<NonNull<()>>,
    frame_count: u32,
);
pub type StopProc = extern "C" fn(device: NonNull<Device>);

#[repr(transparent)]
pub struct DeviceConfig(sys::ma_device_config);

impl DeviceConfig {
    pub fn new(device_type: DeviceType) -> DeviceConfig {
        DeviceConfig(unsafe { sys::ma_device_config_init(device_type as _) })
    }

    pub fn device_type(&self) -> DeviceType {
        DeviceType::from_c(self.0.deviceType)
    }

    pub fn set_device_type(&mut self, device_type: DeviceType) {
        self.0.deviceType = device_type as _;
    }

    pub fn sample_rate(&self) -> u32 {
        self.0.sampleRate
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.0.sampleRate = sample_rate
    }

    pub fn period_size_in_frames(&self) -> u32 {
        self.0.periodSizeInFrames
    }

    pub fn set_period_size_in_frames(&mut self, size: u32) {
        self.0.periodSizeInFrames = size;
    }

    pub fn period_size_in_milliseconds(&self) -> u32 {
        self.0.periodSizeInMilliseconds
    }

    pub fn set_period_size_in_milliseconds(&mut self, size: u32) {
        self.0.periodSizeInMilliseconds = size;
    }

    pub fn periods(&self) -> u32 {
        self.0.periods
    }

    pub fn set_periods(&mut self, periods: u32) {
        self.0.periods = periods
    }

    pub fn peformance_profile(&self) -> PerformanceProfile {
        PerformanceProfile::from_c(self.0.performanceProfile)
    }

    pub fn set_performance_profile(&mut self, profile: PerformanceProfile) {
        self.0.performanceProfile = profile as _
    }

    pub fn no_pre_zeroed_output_buffer(&self) -> bool {
        from_bool32(self.0.noPreZeroedOutputBuffer)
    }

    pub fn set_no_pre_zeroed_output_buffer(&mut self, value: bool) {
        self.0.noPreZeroedOutputBuffer = to_bool32(value);
    }

    pub fn no_clip(&self) -> bool {
        from_bool32(self.0.noClip)
    }

    pub fn set_no_clip(&mut self, no_clip: bool) {
        self.0.noClip = to_bool32(no_clip);
    }

    #[inline]
    pub fn playback(&self) -> &DeviceConfigPlayback {
        unsafe { std::mem::transmute(&self.0.playback) }
    }

    #[inline]
    pub fn playback_mut(&mut self) -> &mut DeviceConfigPlayback {
        unsafe { std::mem::transmute(&mut self.0.playback) }
    }

    #[inline]
    pub fn capture(&self) -> &DeviceConfigCapture {
        unsafe { std::mem::transmute(&self.0.capture) }
    }

    #[inline]
    pub fn capture_mut(&mut self) -> &mut DeviceConfigCapture {
        unsafe { std::mem::transmute(&mut self.0.capture) }
    }

    #[inline]
    pub fn resampling(&self) -> &DeviceConfigResampling {
        unsafe { std::mem::transmute(&self.0.resampling) }
    }

    #[inline]
    pub fn resampling_mut(&mut self) -> &mut DeviceConfigResampling {
        unsafe { std::mem::transmute(&mut self.0.resampling) }
    }

    /// Sets the data callback for this device config.
    ///
    /// **IMPORTANT** The function passed in here must be cloneable because each device that uses
    /// this config will create and use a clone of the given function and its environment. In order
    /// to share a variable between distinct device instances they have to be wrapped in some sort
    /// of cloneable thread-safe struct like an Arc.
    pub fn set_data_callback<F>(&mut self, callback: F)
    where
        F: FnMut(NonNull<Device>, Option<NonNull<()>>, Option<NonNull<()>>, u32)
            + Send
            + Sync
            + Clone
            + 'static,
    {
        let user_data = self.ensure_user_data();
        unsafe {
            (*user_data).data_callback_factory = Some(Box::new(move || Box::new(callback.clone())));
        }

        // This is set in here instead of the Device's initialization code because overwritting
        // the value of the data callback after device initialization is not allowed by miniaudio's
        // API.
        self.0.dataCallback = Some(device_data_callback_trampoline);
    }

    /// Sets the stop callback for this device config.
    ///
    /// **IMPORTANT** The function passed in here must be cloneable because each device that uses
    /// this config will create and use a clone of the given function and its environment. In order
    /// to share a variable between distinct device instances they have to be wrapped in some sort
    /// of cloneable thread-safe struct like an Arc.
    pub fn set_stop_callback<F>(&mut self, callback: F)
    where
        F: FnMut(NonNull<Device>) + Clone + Send + Sync + 'static,
    {
        let user_data = self.ensure_user_data();
        unsafe {
            (*user_data).stop_callback_factory = Some(Box::new(move || Box::new(callback.clone())));
        }

        // This is set in here instead of the Device's initialization code because overwritting
        // the value of the stop callback after device initialization is not allowed by miniaudio's
        // API.
        self.0.stopCallback = Some(device_stop_callback_trampoline);
    }

    /// This will ensure that user data is initialized and return an unsafe mutable pointer to it.
    fn ensure_user_data(&mut self) -> *mut DeviceConfigUserData {
        if self.0.pUserData.is_null() {
            self.0.pUserData = Box::into_raw(Box::new(DeviceConfigUserData {
                data_callback_factory: None,
                stop_callback_factory: None,
            })) as *mut _;
        }
        self.0.pUserData.cast()
    }
    // FIXME implement getters/setters for wasapi/alsa/pulse config structs
}

pub struct DeviceConfigUserData {
    data_callback_factory: Option<
        Box<
            dyn Fn() -> Box<
                dyn FnMut(NonNull<Device>, Option<NonNull<()>>, Option<NonNull<()>>, u32),
            >,
        >,
    >,
    stop_callback_factory: Option<Box<dyn Fn() -> Box<dyn FnMut(NonNull<Device>)>>>,
}

// FIXME it might be better to just set the callbacks to some noop functions by default
// to save ourselves the extra in the audio callback code.
pub struct DeviceUserData {
    data_callback:
        Option<Box<dyn FnMut(NonNull<Device>, Option<NonNull<()>>, Option<NonNull<()>>, u32)>>,
    stop_callback: Option<Box<dyn FnMut(NonNull<Device>)>>,
}

unsafe extern "C" fn device_data_callback_trampoline(
    device_ptr: *mut sys::ma_device,
    output_ptr: *mut c_void,
    input_ptr: *const c_void,
    frame_count: u32,
) {
    if let Some(device) = NonNull::new(device_ptr.cast::<Device>()) {
        let output = NonNull::new(output_ptr as *mut ());
        let input = NonNull::new(input_ptr as *mut ());
        let user_data = (*device.as_ptr()).0.pUserData.cast::<DeviceUserData>();
        if user_data.is_null() {
            return;
        }

        if let Some(ref mut data_callback) = (*user_data).data_callback {
            (data_callback)(device, output, input, frame_count);
        }
    }
}

unsafe extern "C" fn device_stop_callback_trampoline(device_ptr: *mut sys::ma_device) {
    if let Some(device) = NonNull::new(device_ptr.cast::<Device>()) {
        let user_data = (*device.as_ptr()).0.pUserData.cast::<DeviceUserData>();
        if user_data.is_null() {
            return;
        }
        if let Some(ref mut stop_callback) = (*user_data).stop_callback {
            (stop_callback)(device);
        }
    }
}

impl Drop for DeviceConfig {
    fn drop(&mut self) {
        let user_data = self.0.pUserData;
        if !user_data.is_null() {
            unsafe { Box::from_raw(user_data.cast::<DeviceConfigUserData>()) }; // drop it
        }
    }
}

#[repr(transparent)]
pub struct DeviceConfigPlayback(MADeviceConfigPlayback);

impl DeviceConfigPlayback {
    pub fn device_id(&self) -> Option<NonNull<DeviceId>> {
        unsafe { std::mem::transmute(self.0.pDeviceID) }
    }

    pub fn set_device_id(&mut self, device_id: Option<NonNull<DeviceId>>) {
        self.0.pDeviceID = device_id
            .map(|id| id.cast().as_ptr())
            .unwrap_or(ptr::null_mut());
    }

    pub fn format(&self) -> Format {
        Format::from_c(self.0.format)
    }

    pub fn set_format(&mut self, format: Format) {
        self.0.format = format as _;
    }

    pub fn channels(&self) -> u32 {
        self.0.channels
    }

    pub fn set_channels(&mut self, channels: u32) {
        self.0.channels = channels;
    }

    pub fn channel_map(&self) -> &[Channel; sys::MA_MAX_CHANNELS as usize] {
        unsafe { std::mem::transmute(&self.0.channelMap) }
    }

    pub fn channel_map_mut(&mut self) -> &mut [Channel; sys::MA_MAX_CHANNELS as usize] {
        unsafe { std::mem::transmute(&mut self.0.channelMap) }
    }

    pub fn share_mode(&self) -> ShareMode {
        ShareMode::from_c(self.0.shareMode)
    }

    pub fn set_share_mode(&mut self, share_mode: ShareMode) {
        self.0.shareMode = share_mode as _
    }
}

#[repr(transparent)]
pub struct DeviceConfigCapture(MADeviceConfigCapture);

impl DeviceConfigCapture {
    pub fn device_id(&self) -> Option<NonNull<DeviceId>> {
        unsafe { std::mem::transmute(self.0.pDeviceID) }
    }

    pub fn set_device_id(&mut self, device_id: Option<NonNull<DeviceId>>) {
        self.0.pDeviceID = device_id
            .map(|id| id.cast().as_ptr())
            .unwrap_or(ptr::null_mut());
    }

    pub fn format(&self) -> Format {
        Format::from_c(self.0.format)
    }

    pub fn set_format(&mut self, format: Format) {
        self.0.format = format as _;
    }

    pub fn channels(&self) -> u32 {
        self.0.channels
    }

    pub fn set_channels(&mut self, channels: u32) {
        self.0.channels = channels;
    }

    pub fn channel_map(&self) -> &[Channel; sys::MA_MAX_CHANNELS as usize] {
        unsafe { std::mem::transmute(&self.0.channelMap) }
    }

    pub fn channel_map_mut(&mut self) -> &mut [Channel; sys::MA_MAX_CHANNELS as usize] {
        unsafe { std::mem::transmute(&mut self.0.channelMap) }
    }

    pub fn share_mode(&self) -> ShareMode {
        ShareMode::from_c(self.0.shareMode)
    }

    pub fn set_share_mode(&mut self, share_mode: ShareMode) {
        self.0.shareMode = share_mode as _
    }
}

#[repr(transparent)]
pub struct DeviceConfigResampling(MADeviceConfigResampling);

impl DeviceConfigResampling {
    pub fn algorithm(&self) -> ResampleAlgorithm {
        ResampleAlgorithm::from_c(self.0.algorithm)
    }

    pub fn set_algorithm(&mut self, algorithm: ResampleAlgorithm) {
        self.0.algorithm = algorithm as _;
    }

    pub fn linear_lpf_poles(&self) -> u32 {
        self.0.linear.lpfPoles
    }

    pub fn set_linear_lpf_poles(&mut self, lpf_poles: u32) {
        self.0.linear.lpfPoles = lpf_poles;
    }

    pub fn speex_quality(&self) -> i32 {
        self.0.speex.quality as i32
    }

    pub fn set_speex_quality(&mut self, quality: i32) {
        self.0.speex.quality = quality as _;
    }
}

#[repr(transparent)]
pub struct ContextConfig(sys::ma_context_config);

impl ContextConfig {
    /// Initializes a ContextConfig object.
    pub fn new() -> ContextConfig {
        ContextConfig(unsafe { sys::ma_context_config_init() })
    }

    // FIXME implement some stuff for context config.
    //       I just don't need it at the moement, so...
}

#[repr(transparent)]
pub struct Context(sys::ma_context);

impl Context {
    /// - `backends` - A list of backends to try initializing, in priority order. Can be None, in which case it
    /// uses default priority order.
    ///
    /// - `config` - Optional context configuration.
    pub fn alloc(
        backends: Option<&[Backend]>,
        config: Option<&ContextConfig>,
    ) -> Result<Arc<Context>, Error> {
        let context = Arc::new(MaybeUninit::<sys::ma_context>::uninit());

        let result = unsafe {
            sys::ma_context_init(
                backends
                    .map(|b| b.as_ptr() as *const _)
                    .unwrap_or(ptr::null()),
                backends.map(|b| b.len() as u32).unwrap_or(0),
                config.map(|c| &c.0 as *const _).unwrap_or(ptr::null()),
                Arc::deref(&context).as_ptr() as *mut _,
            )
        };

        map_result!(result, unsafe { std::mem::transmute(context) })
    }

    pub fn backend(&self) -> Backend {
        Backend::from_c(self.0.backend)
    }

    pub fn thread_priority(&self) -> ThreadPriority {
        ThreadPriority::from_c(self.0.threadPriority)
    }

    pub fn device_info_capacity(&self) -> u32 {
        self.0.deviceInfoCapacity
    }

    pub fn playback_device_info_count(&self) -> u32 {
        self.0.playbackDeviceInfoCount
    }

    pub fn capture_device_info_count(&self) -> u32 {
        self.0.captureDeviceInfoCount
    }

    pub fn playback_device_infos(&self) -> &[DeviceInfo] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.pDeviceInfos as _,
                self.0.playbackDeviceInfoCount as usize,
            )
        }
    }

    pub fn capture_device_info(&self) -> &[DeviceInfo] {
        unsafe {
            std::slice::from_raw_parts(
                self.0
                    .pDeviceInfos
                    .add(self.0.playbackDeviceInfoCount as usize) as _,
                self.0.captureDeviceInfoCount as usize,
            )
        }
    }

    pub fn is_backend_asynchronous(&self) -> bool {
        from_bool32(self.0.isBackendAsynchronous())
    }

    /// Retrieves basic information about every active playback and capture device. This function
    /// will allocate memory internally for device lists.
    /// This function will not call the closure if an error occurred.
    pub fn with_devices<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&[DeviceInfo], &[DeviceInfo]),
    {
        let mut playback_ptr: *mut sys::ma_device_info = ptr::null_mut();
        let mut playback_count: u32 = 0;

        let mut capture_ptr: *mut sys::ma_device_info = ptr::null_mut();
        let mut capture_count = 0;

        unsafe {
            let result = sys::ma_context_get_devices(
                &self.0 as *const _ as *mut _,
                &mut playback_ptr,
                &mut playback_count,
                &mut capture_ptr,
                &mut capture_count,
            );

            if Error::is_c_error(result) {
                return Err(Error::from_c_error(result));
            }

            f(
                std::slice::from_raw_parts(
                    std::mem::transmute::<_, *mut DeviceInfo>(playback_ptr),
                    playback_count as usize,
                ),
                std::slice::from_raw_parts(
                    std::mem::transmute::<_, *mut DeviceInfo>(capture_ptr),
                    capture_count as usize,
                ),
            );
        }

        return Ok(());
    }

    /// Retrieves basic information about every active playback device. This function
    /// will allocate memory internally for device lists.
    /// This function will not call the closure if an error occurred.
    pub fn with_playback_devices<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&[DeviceInfo]),
    {
        let mut playback_ptr: *mut sys::ma_device_info = ptr::null_mut();
        let mut playback_count: u32 = 0;

        unsafe {
            let result = sys::ma_context_get_devices(
                &self.0 as *const _ as *mut _,
                &mut playback_ptr,
                &mut playback_count,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            if Error::is_c_error(result) {
                return Err(Error::from_c_error(result));
            }

            f(std::slice::from_raw_parts(
                std::mem::transmute::<_, *mut DeviceInfo>(playback_ptr),
                playback_count as usize,
            ));
        }

        return Ok(());
    }

    /// Retrieves basic information about every active capture device. This function
    /// will allocate memory internally for device lists.
    /// This function will not call the closure if an error occurred.
    pub fn with_capture_devices<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&[DeviceInfo]),
    {
        let mut capture_ptr: *mut sys::ma_device_info = ptr::null_mut();
        let mut capture_count = 0;

        unsafe {
            let result = sys::ma_context_get_devices(
                &self.0 as *const _ as *mut _,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut capture_ptr,
                &mut capture_count,
            );

            if Error::is_c_error(result) {
                return Err(Error::from_c_error(result));
            }

            f(std::slice::from_raw_parts(
                std::mem::transmute::<_, *mut DeviceInfo>(capture_ptr),
                capture_count as usize,
            ));
        }

        return Ok(());
    }

    pub fn enumerate_devices<F>(&self, mut callback: F) -> Result<(), Error>
    where
        F: for<'r, 's> FnMut(&'r Context, DeviceType, &'s DeviceInfo) -> bool,
    {
        let mut callback_ptr: &mut dyn FnMut(&Context, DeviceType, &DeviceInfo) -> bool =
            &mut callback;
        let callback_ptr_ptr = &mut callback_ptr;

        let result = unsafe {
            sys::ma_context_enumerate_devices(
                &self.0 as *const _ as *mut _,
                Some(enumerate_devices_inner_trampoline),
                callback_ptr_ptr as *mut _ as *mut c_void,
            )
        };

        return Error::from_c_result(result);

        unsafe extern "C" fn enumerate_devices_inner_trampoline(
            context: *mut sys::ma_context,
            device_type: sys::ma_device_type,
            info: *const sys::ma_device_info,
            udata: *mut c_void,
        ) -> u32 {
            let real_callback =
                udata as *mut &mut dyn FnMut(&Context, DeviceType, &DeviceInfo) -> bool;
            let b = (*real_callback)(
                (context as *mut Context).as_mut().unwrap(),
                DeviceType::from_c(device_type),
                (info as *const DeviceInfo).as_ref().unwrap(),
            );

            to_bool32(b)
        }
    }

    // FIXME implement ma_context_enumerate_devices

    // FIXME implement user data / callbacks
    // See `DeviceConfig` for how to do that.
}

impl Drop for Context {
    fn drop(&mut self) {
        Error::from_c_result(unsafe { sys::ma_context_uninit(&mut self.0) })
            .expect("failed to uninit context");
    }
}

#[repr(transparent)]
pub struct Device(sys::ma_device);

impl Device {
    pub fn alloc(
        context: Option<Arc<Context>>,
        config: &DeviceConfig,
    ) -> Result<Arc<Device>, Error> {
        let device = Arc::new(MaybeUninit::<sys::ma_device>::uninit());

        let result = unsafe {
            sys::ma_device_init(
                context
                    .map(|c| Arc::into_raw(c) as *mut _)
                    .unwrap_or(ptr::null_mut()),
                config as *const DeviceConfig as *const _,
                Arc::deref(&device).as_ptr() as *mut _,
            )
        };

        unsafe { (*(Arc::deref(&device).as_ptr() as *mut Device)).create_device_user_data() };

        map_result!(result, unsafe { std::mem::transmute(device) })
    }

    fn create_device_user_data(&mut self) {
        if !self.0.pUserData.is_null() {
            let config_user_data = self.0.pUserData.cast::<DeviceConfigUserData>();

            let data_callback = unsafe {
                ((*config_user_data).data_callback_factory)
                    .as_ref()
                    .map(|f| (f)())
            };
            let stop_callback = unsafe {
                ((*config_user_data).stop_callback_factory)
                    .as_ref()
                    .map(|f| (f)())
            };

            self.0.pUserData = Box::into_raw(Box::new(DeviceUserData {
                data_callback,
                stop_callback,
            })) as *mut _;
        }
    }

    /// Starts the device. For playback devices this begins playback. For capture devices this
    /// begins recording.
    /// Use `stop` to stop this device.
    ///
    /// **WARNING** This should not be called from a callback.
    pub fn start(&self) -> Result<(), Error> {
        Error::from_c_result(unsafe { sys::ma_device_start(&self.0 as *const _ as *mut _) })
    }

    /// Stops this device. For playback devices this stops playback. For capture devices this stops
    /// recording. Use `start` to start this device again.
    ///
    /// **WARNING** This should not be called from a callback.
    pub fn stop(&self) -> Result<(), Error> {
        Error::from_c_result(unsafe { sys::ma_device_stop(&self.0 as *const _ as *mut _) })
    }

    /// Returns true if this device has started.
    pub fn is_started(&self) -> bool {
        from_bool32(unsafe { sys::ma_device_is_started(&self.0 as *const _ as *mut _) })
    }

    /// Sets the master volume factor for the device.
    ///
    /// The volume factor must be between 0 (silence) and 1 (full volume). Use `set_master_gain_db()` to use decibel notation, where 0 is full volume and
    /// values less than 0 decreases the volume.
    ///
    /// Callback Safety
    /// ---------------
    /// Safe. If you set the volume in the data callback, that data written to the output buffer will have the new volume applied.
    pub fn set_master_volume(&self, volume: f32) -> Result<(), Error> {
        Error::from_c_result(unsafe {
            sys::ma_device_set_master_volume(&self.0 as *const _ as *mut _, volume)
        })
    }

    /// Retrieves the master volume factor for the device.
    pub fn get_master_volume(&self) -> Result<f32, Error> {
        let mut out = 0.0;
        map_result!(
            unsafe { sys::ma_device_get_master_volume(&self.0 as *const _ as *mut _, &mut out) },
            out
        )
    }

    /// Sets the master volume for the device as gain in decibels.
    ///
    /// A gain of 0 is full volume, whereas a gain of < 0 will decrease the volume.
    ///
    /// The volume factor must be between 0 (silence) and 1 (full volume). Use `set_master_gain_db()` to use decibel notation, where 0 is full volume and
    /// values less than 0 decreases the volume.
    ///
    /// Callback Safety
    /// ---------------
    /// Safe. If you set the volume in the data callback, that data written to the output buffer will have the new volume applied.
    pub fn set_master_gain_db(&self, gain_db: f32) -> Result<(), Error> {
        Error::from_c_result(unsafe {
            sys::ma_device_set_master_gain_db(&self.0 as *const _ as *mut _, gain_db)
        })
    }

    /// Retrieves the master gain in decibels.
    pub fn get_master_gain_db(&self) -> Result<f32, Error> {
        let mut out = 0.0;
        map_result!(
            unsafe { sys::ma_device_get_master_gain_db(&self.0 as *const _ as *mut _, &mut out) },
            out
        )
    }

    /// This is set to true if the context was created by and is managed by the device.
    /// If this is false, then the context is created by the user and should be cleaned by on the
    /// Rust side.
    fn is_owner_of_context(&self) -> bool {
        from_bool32(self.0.isOwnerOfContext())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // We have to copy these before the struct is zeroed.
        let is_owner_of_context = self.is_owner_of_context();
        let context_ptr = self.0.pContext;
        let user_data = self.0.pUserData;

        unsafe { sys::ma_device_uninit(&mut self.0) };

        // We drop this AFTER uninit so that the stop callback can execute correctly.
        if !user_data.is_null() {
            unsafe { Box::from_raw(user_data.cast::<DeviceUserData>()) }; // drop it
        }

        // We only decrement the context ref count if we own it and now the device.
        if !is_owner_of_context && !context_ptr.is_null() {
            let context_arc = unsafe { Arc::from_raw(context_ptr as *const _) };
            self.0.pContext = ptr::null_mut();
            drop(context_arc);
        }
    }
}
