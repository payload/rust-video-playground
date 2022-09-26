pub use objc::*;
pub use objc_foundation::*;
pub use objc_id::*;

object_struct!(AVCaptureDevice);

pub struct AVMediaTypeVideo;

impl AVMediaType for AVMediaTypeVideo {
    fn to_nsstring(&self) -> Id<NSString> {
        NSString::from_str("vide")
    }
}

pub trait AVMediaType {
    fn to_nsstring(&self) -> Id<NSString>;
}

#[allow(non_snake_case)]
impl AVCaptureDevice {
    pub fn devicesWithMediaType(mediaType: impl AVMediaType) -> Vec<Id<Self>> {
        // NSArray *devices = [AVCaptureDevice devicesWithMediaType:AVMediaTypeVideo];
        let AVCaptureDevice = class!(AVCaptureDevice);
        let mediaType = mediaType.to_nsstring();
        NSArray::into_vec(unsafe {
            Id::from_ptr(msg_send!(AVCaptureDevice, devicesWithMediaType: mediaType))
        })
    }

    pub fn localizedName(&self) -> Id<NSString> {
        unsafe { Id::from_ptr(msg_send!(self, localizedName)) }
    }
}
