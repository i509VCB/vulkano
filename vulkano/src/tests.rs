// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#![cfg(test)]

/// Creates an instance or returns if initialization fails.
macro_rules! instance {
    () => {{
        use crate::instance;
        use crate::Version;

        match instance::Instance::new(
            None,
            Version::V1_1,
            &instance::InstanceExtensions::none(),
            None,
        ) {
            Ok(i) => i,
            Err(_) => return,
        }
    }};
}

/// Creates a device and a queue for graphics operations.
macro_rules! gfx_dev_and_queue {
    ($($feature:ident),*) => ({
        use crate::instance;
        use crate::device::Device;
        use crate::device::DeviceExtensions;
        use crate::device::Features;

        let instance = instance!();

        let physical = match instance::PhysicalDevice::enumerate(&instance).next() {
            Some(p) => p,
            None => return
        };

        let queue = match physical.queue_families().find(|q| q.supports_graphics()) {
            Some(q) => q,
            None => return
        };

        let extensions = DeviceExtensions::none();

        let features = Features {
            $(
                $feature: true,
            )*
            .. Features::none()
        };

        // If the physical device doesn't support the requested features, just return.
        if !physical.supported_features().superset_of(&features) {
            return;
        }

        let (device, mut queues) = match Device::new(physical, &features,
                                                     &extensions, [(queue, 0.5)].iter().cloned())
        {
            Ok(r) => r,
            Err(_) => return
        };

        (device, queues.next().unwrap())
    });
}

macro_rules! assert_should_panic {
    ($msg:expr, $code:block) => {{
        let res = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| $code));

        match res {
            Ok(_) => panic!("Test expected to panic but didn't"),
            Err(err) => {
                if let Some(msg) = err.downcast_ref::<String>() {
                    assert!(msg.contains($msg));
                } else if let Some(&msg) = err.downcast_ref::<&str>() {
                    assert!(msg.contains($msg));
                } else {
                    panic!("Couldn't decipher the panic message of the test")
                }
            }
        }
    }};

    ($code:block) => {{
        let res = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| $code));

        match res {
            Ok(_) => panic!("Test expected to panic but didn't"),
            Err(_) => {}
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::instance::{Instance, InstanceExtensions, RawInstanceExtensions, PhysicalDevice};
    use crate::{Version, VulkanObject};
    use std::ffi::CString;
    use crate::device::{RawDeviceExtensions, Device, Features};
    use std::{ptr, mem};
    use ash::vk::PhysicalDeviceProperties;
    use std::os::raw::c_void;
    use std::mem::MaybeUninit;

    #[test]
    fn test() {
        eprintln!("Test");
        let extensions = InstanceExtensions {
            khr_get_physical_device_properties2: true,
            ..InstanceExtensions::none()
        };

        let instance = Instance::new(
            None,
            Version::V1_2,
            &extensions,
            None,
            //["VK_LAYER_KHRONOS_validation"],
        ).unwrap();

        let count = PhysicalDevice::enumerate(&instance).count();
        println!("{}", count);

        let physical_device = PhysicalDevice::enumerate(&instance).nth(1).unwrap();

        println!("{}", physical_device.name());

        let extensions = RawDeviceExtensions::supported_by_device(physical_device);
        let queue_family = physical_device.queue_families().find(|q| q.supports_graphics()).unwrap();

        {
            #[derive(Debug)]
            #[repr(C)]
            struct PhysicalDeviceDrmPropertiesEXT {
                s_type: ash::vk::StructureType,
                p_next: *mut c_void,
                has_primary: ash::vk::Bool32,
                has_render: ash::vk::Bool32,
                primary_major: i64,
                primary_minor: i64,
                render_major: i64,
                render_minor: i64,
            }

            let internal_physical_device = physical_device.internal_object();
            let ext = unsafe { PhysicalDeviceDrmPropertiesEXT {
                s_type: ash::vk::StructureType::from_raw(1000353000), // magic value
                ..mem::zeroed()
            } };
            let mut ext = MaybeUninit::new(ext);

            let mut properties = unsafe { ash::vk::PhysicalDeviceProperties2 {
                s_type: ash::vk::StructureType::PHYSICAL_DEVICE_PROPERTIES_2_KHR,
                p_next: (&mut ext).as_mut_ptr() as *mut c_void,
                ..mem::zeroed()
            } };

            unsafe { physical_device.instance()
                .clone()
                .fns()
                .khr_get_physical_device_properties2
                .get_physical_device_properties2_khr(
                    internal_physical_device,
                    &mut properties
                );
            }

            eprintln!("{:?}", properties);

            let ext = properties.p_next as *mut PhysicalDeviceDrmPropertiesEXT;

            if ext.is_null() {
                panic!()
            }

            eprintln!("{:?}", unsafe { ext.as_ref().unwrap() });

            // let (device, queues) = Device::new(
            //     physical_device,
            //     &Features::none(),
            //     extensions.clone(),
            //     [(queue_family, 0.5)].iter().cloned()
            // ).unwrap();
        }
    }
}
