use core::iter;

use super::mmio::virtio_probe_mmio;
use super::transport_pci::PciTransport;
use super::virtio_impl::HalImpl;
use crate::driver::base::device::DeviceId;
use crate::driver::block::virtio_blk::{virtio_blk, virtio_blk_driver_init, VIRTIO_BLK_DRIVER};
use crate::driver::net::virtio_net::virtio_net;
use crate::driver::pci::pci::{
    get_pci_device_structures_mut_by_vendor_id, PciDeviceStructure, PciDeviceStructureGeneralDevice, PCI_CAP_ID_MSIX, PCI_DEVICE_LINKEDLIST
};
use crate::driver::virtio::transport::VirtIOTransport;
use crate::libs::rwlock::RwLockWriteGuard;

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{boxed::Box, collections::LinkedList};
use log::{debug, error, warn};
use virtio_drivers::transport::{DeviceType, Transport};

///@brief 寻找并加载所有virtio设备的驱动（目前只有virtio-net，但其他virtio设备也可添加）
pub fn virtio_probe() {
    #[cfg(not(target_arch = "riscv64"))]
    virtio_probe_pci();
    virtio_probe_mmio();
    // virtio_blk_driver_init().unwrap();
}

#[allow(dead_code)]
fn virtio_probe_pci() {
    let mut list = PCI_DEVICE_LINKEDLIST.write();
    let mut virtio_list = virtio_device_search(&mut list);
    let mut add=0;
    virtio_list.reverse();
    for virtio_device in virtio_list {
        let dev_id = virtio_device.common_header.device_id;
        let dev_id = DeviceId::new(None, Some(format!("{dev_id}"))).unwrap();
        debug!("probing pci virtio device-cpt:{:?},vendor:{:?},device_id:{:?}",virtio_device.capabilities_pointer,virtio_device.common_header.vendor_id,virtio_device.common_header.device_id);
        
        // for i in virtio_device.capabilities().unwrap(){
        //     debug!("CapabilityInfo:{:?}",i);
        // }
        // let mut a=0;
        // continue;
        // let mut compatible_flag=false;
        // for i in virtio_device.capabilities().unwrap(){
        //     if i.id==PCI_CAP_ID_MSIX{
        //        compatible_flag=true; 
        //     }
        // }
        // if !compatible_flag {
        //     break;
        // }
        match PciTransport::new::<HalImpl>(virtio_device, dev_id.clone(),add) {
            
            Ok(mut transport) => {
                if virtio_device.common_header.vendor_id==6900&&virtio_device.common_header.device_id==4097{
                    let standard_device = virtio_device.as_standard_device_mut().unwrap();
                    let a=standard_device.msix_capability_offset();
                    // for i in 0..1500{
                        debug!("4097's msix offset={:?}",a);
                    // }
                    debug!(
                        "Detected virtio PCI device with device type {:?}, features {:#018x}",
                        transport.device_type(),
                        transport.read_device_features(),
                    );
                    loop{}
                }
                debug!(
                    "Detected virtio PCI device with device type {:?}, features {:#018x}",
                    transport.device_type(),
                    transport.read_device_features(),
                );
                let transport = VirtIOTransport::Pci(transport);
                
                virtio_device_init(transport, dev_id);
            }
            Err(err) => {
                if virtio_device.common_header.vendor_id==6900&&virtio_device.common_header.device_id==4097{
                    let standard_device = virtio_device.as_standard_device_mut().unwrap();
                    let a=standard_device.msix_capability_offset();
                    // for i in 0..1500{
                        debug!("4097's msix offset={:?}",a);
                    // }
                    error!("Pci transport create failed because of error: {}", err);
                    loop{}
                }
                error!("Pci transport create failed because of error: {}", err);
                
            }
        }
       
        add+=1;
    }
    // loop{}
}

///@brief 为virtio设备寻找对应的驱动进行初始化
pub(super) fn virtio_device_init(transport: VirtIOTransport, dev_id: Arc<DeviceId>) {
    match transport.device_type() {
        DeviceType::Block => virtio_blk(transport, dev_id),
        DeviceType::GPU => {
            warn!("Not support virtio_gpu device for now");
        }
        DeviceType::Input => {
            warn!("Not support virtio_input device for now");
        }
        DeviceType::Network => virtio_net(transport, dev_id),
        t => {
            warn!("Unrecognized virtio device: {:?}", t);
        }
    }
}

/// # virtio_device_search - 在给定的PCI设备列表中搜索符合特定标准的virtio设备
///
/// 该函数搜索一个PCI设备列表，找到所有由特定厂商ID（0x1AF4）和设备ID范围（0x1000至0x103F）定义的virtio设备。
///
/// ## 参数
///
/// - list: &'a mut RwLockWriteGuard<'_, LinkedList<Box<dyn PciDeviceStructure>>> - 一个可写的PCI设备结构列表的互斥锁。
///
/// ## 返回值
///
/// 返回一个包含所有找到的virtio设备的数组
fn virtio_device_search<'a>(
    list: &'a mut RwLockWriteGuard<'_, LinkedList<Box<dyn PciDeviceStructure>>>,
) -> Vec<&'a mut PciDeviceStructureGeneralDevice> {
    let mut virtio_list = Vec::new();
    let result = get_pci_device_structures_mut_by_vendor_id(list, 0x1AF4);

    for device in result {
        let standard_device = device.as_standard_device_mut().unwrap();
        let header = &standard_device.common_header;
        if header.device_id >= 0x1000 && header.device_id <= 0x103F {
            virtio_list.push(standard_device);
        }
    }

    return virtio_list;
}
