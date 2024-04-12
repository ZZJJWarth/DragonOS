use alloc::{string::{String, ToString}, sync::{Arc, Weak}, vec::Vec};

use crate::{driver::{base::{device::{bus::Bus, driver::Driver, Device, IdTable}, kobject::{KObjType, KObject, KObjectState, LockedKObjectState}, kset::KSet}, pci_driver::{dev_id::PciDeviceID, pci_driver::PciDriver, test::pt_driver::InnerPciDriver}}, filesystem::kernfs::KernFSInode, libs::rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard}};


#[derive(Debug)]
pub struct VirtIODriver{
    inner: RwLock<InnerPciDriver>,
    kobj_state: LockedKObjectState,
}

impl VirtIODriver {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(InnerPciDriver {
                ktype: None,
                kset: None,
                parent: None,
                kernfs_inode: None,
                devices: Vec::new(),
                bus: None,
                self_ref: Weak::new(),
                locked_dynid_list: Vec::new(),
            }),

            kobj_state: LockedKObjectState::new(None),
        }
    }
}

impl PciDriver for VirtIODriver {
    fn add_dynid(
        &mut self,
        id: crate::driver::pci_driver::dev_id::PciDeviceID,
    ) -> Result<(), system_error::SystemError> {
        self.inner.write().insert_id(id);
        Ok(())
    }

    fn locked_dynid_list(&self) -> Option<Vec<Arc<PciDeviceID>>> {
        Some(self.inner.read().id_list().clone())
    }

    fn probe(
        &self,
        device: &Arc<dyn crate::driver::pci_driver::pci_device::PciDevice>,
        id: &crate::driver::pci_driver::dev_id::PciDeviceID,
    ) -> Result<(), system_error::SystemError> {
        Ok(())
    }

    fn remove(
        &self,
        device: &Arc<dyn crate::driver::pci_driver::pci_device::PciDevice>,
    ) -> Result<(), system_error::SystemError> {
        Ok(())
    }

    fn resume(
        &self,
        device: &Arc<dyn crate::driver::pci_driver::pci_device::PciDevice>,
    ) -> Result<(), system_error::SystemError> {
        Ok(())
    }

    fn shutdown(
        &self,
        device: &Arc<dyn crate::driver::pci_driver::pci_device::PciDevice>,
    ) -> Result<(), system_error::SystemError> {
        Ok(())
    }

    fn suspend(
        &self,
        device: &Arc<dyn crate::driver::pci_driver::pci_device::PciDevice>,
    ) -> Result<(), system_error::SystemError> {
        Ok(())
    }
}

impl Driver for VirtIODriver {
    fn id_table(&self) -> Option<IdTable> {
        Some(IdTable::new("VirtioDriver".to_string(), None))
    }

    fn devices(&self) -> Vec<Arc<dyn Device>> {
        self.inner.read().devices.clone()
    }

    fn add_device(&self, device: Arc<dyn Device>) {
        let mut guard = self.inner.write();
        // check if the device is already in the list
        if guard.devices.iter().any(|dev| Arc::ptr_eq(dev, &device)) {
            return;
        }

        guard.devices.push(device);
    }

    fn delete_device(&self, device: &Arc<dyn Device>) {
        let mut guard = self.inner.write();
        guard.devices.retain(|dev| !Arc::ptr_eq(dev, device));
    }

    fn set_bus(&self, bus: Option<Weak<dyn Bus>>) {
        self.inner.write().bus = bus;
    }

    fn bus(&self) -> Option<Weak<dyn Bus>> {
        self.inner.read().bus.clone()
    }
}

impl KObject for VirtIODriver {
    fn as_any_ref(&self) -> &dyn core::any::Any {
        self
    }

    fn set_inode(&self, inode: Option<Arc<KernFSInode>>) {
        self.inner.write().kernfs_inode = inode;
    }

    fn inode(&self) -> Option<Arc<KernFSInode>> {
        self.inner.read().kernfs_inode.clone()
    }

    fn parent(&self) -> Option<Weak<dyn KObject>> {
        self.inner.read().parent.clone()
    }

    fn set_parent(&self, parent: Option<Weak<dyn KObject>>) {
        self.inner.write().parent = parent;
    }

    fn kset(&self) -> Option<Arc<KSet>> {
        self.inner.read().kset.clone()
    }

    fn set_kset(&self, kset: Option<Arc<KSet>>) {
        self.inner.write().kset = kset;
    }

    fn kobj_type(&self) -> Option<&'static dyn KObjType> {
        self.inner.read().ktype
    }

    fn set_kobj_type(&self, ktype: Option<&'static dyn KObjType>) {
        self.inner.write().ktype = ktype;
    }

    fn name(&self) -> String {
        "Virtio".to_string()
    }

    fn set_name(&self, _name: String) {
        // do nothing
    }

    fn kobj_state(&self) -> RwLockReadGuard<KObjectState> {
        self.kobj_state.read()
    }

    fn kobj_state_mut(&self) -> RwLockWriteGuard<KObjectState> {
        self.kobj_state.write()
    }

    fn set_kobj_state(&self, state: KObjectState) {
        *self.kobj_state.write() = state;
    }
}