use ash::{Entry, vk};
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0};


unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);

    vk::FALSE
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = Entry::new()?;
    let engine_name = std::ffi::CString::new("Unknown engine")?;
    let app_name = std::ffi::CString::new("The Black Window")?;

    let app_info= vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&engine_name)
        .engine_version(vk::make_version(0, 42, 0))
        .api_version(vk::make_version(1, 0, 106));

    let layer_names: Vec<std::ffi::CString> = vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation")?];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let extension_name_pointers: Vec<*const i8> = vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

    let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION)
        .pfn_user_callback(Some(vulkan_debug_utils_callback));


    let create_info= vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .push_next(&mut debug_create_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers);

    let _instance = unsafe { entry.create_instance(&create_info, None)? };

    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &_instance);

    let _utils_messenger = unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None)? };

    let _phys_devs = unsafe { _instance.enumerate_physical_devices()? };

    let (physical_device, physical_device_properties) = {
        let mut chosen = None;
        for phys_dev in _phys_devs {
            let props = unsafe { _instance.get_physical_device_properties(phys_dev) };
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                chosen = Some((phys_dev, props));
            }
        }
        chosen.unwrap()
    };

    dbg!(&physical_device_properties);

    let _queue_family_properties = unsafe { _instance.get_physical_device_queue_family_properties(physical_device) };

    dbg!(&_queue_family_properties);

    let queue_family_indices = {
        let mut found_graphics_q_index = None;
        let mut found_transfer_q_index = None;
        for (index, queue_family_property) in _queue_family_properties.iter().enumerate() {
            if queue_family_property.queue_count > 0 {
                if queue_family_property.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    found_graphics_q_index = Some(index as u32);
                }

                if queue_family_property.queue_flags.contains(vk::QueueFlags::TRANSFER) &&
                    (found_transfer_q_index.is_none() || !queue_family_property.queue_flags.contains(vk::QueueFlags::GRAPHICS)) {
                    found_transfer_q_index = Some(index as u32);
                }
            }
        }

        (found_graphics_q_index.unwrap(), found_transfer_q_index.unwrap())
    };

    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_indices.0)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_indices.1)
            .queue_priorities(&priorities)
            .build(),
    ];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layer_name_pointers);
    let _logical_device =  unsafe { _instance.create_device(physical_device, &device_create_info, None)? };
    let _graphics_queue = unsafe { _logical_device.get_device_queue(queue_family_indices.0, 0) };
    let _transfer_queue = unsafe { _logical_device.get_device_queue(queue_family_indices.1, 0) };
    
    unsafe {
        _logical_device.destroy_device(None);
        debug_utils.destroy_debug_utils_messenger(_utils_messenger, None);
        _instance.destroy_instance(None);
    };

    Ok(())
}