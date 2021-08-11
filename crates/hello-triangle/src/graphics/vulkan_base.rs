pub use crate::graphics::graphics_errors::GraphicsError;
use ash::{vk, Entry};
use ash::{Device, Instance};
use std::error::Error;
use std::ffi::CString;
use winit::window::Window;

pub struct VulkanBase {
    _entry: Entry,
    instance: Instance,
    device: Device,
}

struct QueueFamilyIndices {
    graphics_family_index: Option<u32>,
}

impl QueueFamilyIndices {
    // Checks if values in QueueFamilyIndices are not None
    fn is_complete(&self) -> bool {
        return self.graphics_family_index.is_some();
    }
}

impl VulkanBase {
    pub fn new(window: &Window) -> Result<VulkanBase, Box<dyn Error>> {
        let (_entry, instance) = VulkanBase::create_instance(window)?;

        let (physical_device, queue_family_indices) = VulkanBase::pick_physical_device(&instance)?;

        let device =
            VulkanBase::create_logical_device(&instance, &physical_device, &queue_family_indices)?;

        let _graphics_queue = unsafe {
            device.get_device_queue(queue_family_indices.graphics_family_index.unwrap(), 0)
        };

        Ok(VulkanBase {
            _entry,
            instance,
            device,
        })
    }

    // Creates an ash Instance, which is a light wrapper around a vk::Instance
    fn create_instance(window: &Window) -> Result<(Entry, Instance), Box<dyn Error>> {
        // Specifies extensions
        let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
        let extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        // Loads names into CStrings
        let application_name = CString::new("Hello Triangle").unwrap();
        let engine_name = CString::new("Hello Triangle Engine").unwrap();

        // Creates application info
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&application_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0));

        // Creates instance info
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names_raw);

        // Creats weird wrapper type for accessing cpp vulkan dynamic library, and creates an ash instance inside
        let entry = unsafe { Entry::new()? };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        return Ok((entry, instance));
    }

    // Picks the first valid physical device
    fn pick_physical_device(
        instance: &Instance,
    ) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), Box<dyn Error>> {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };
        for device in physical_devices {
            if let Some(value) = VulkanBase::is_device_suitable(instance, &device) {
                return Ok((device, value));
            }
        }
        Err(Box::new(GraphicsError::NoValidGPU))
    }

    // Checks whether a given physical device is valid
    fn is_device_suitable(
        instance: &Instance,
        device: &vk::PhysicalDevice,
    ) -> Option<QueueFamilyIndices> {
        let queue_family_indices = VulkanBase::find_queue_families(instance, device);

        if queue_family_indices.is_complete() {
            return Some(queue_family_indices);
        }
        None
    }

    // Finds the queue families of a given physical device
    fn find_queue_families(instance: &Instance, device: &vk::PhysicalDevice) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                return QueueFamilyIndices {
                    graphics_family_index: Some(index as u32),
                };
            }
        }

        QueueFamilyIndices {
            graphics_family_index: None,
        }
    }

    // Creates the logical device based on necessary queue families
    fn create_logical_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        indices: &QueueFamilyIndices,
    ) -> Result<Device, Box<dyn Error>> {
        let queue_priorities = [1.0];

        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics_family_index.unwrap())
            .queue_priorities(&queue_priorities)
            .build()];

        let device_create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_info);

        let device =
            unsafe { instance.create_device(*physical_device, &device_create_info, None)? };

        Ok(device)
    }
}

impl Drop for VulkanBase {
    fn drop(&mut self) {
        println!("Cleaning up VulkanBase!");
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
        println!("Cleaned up VulkanBase!");
    }
}
